/*!
# `derive-mmio` - turning structures into MMIO access objects

## Rationale

In C it is very common to create structures that refer to Memory-Mapped I/O (MMIO) peripherals:

```c
typedef volatile struct uart_t {
    uint32_t data;
    const uint32_t status;
} uart_t;

uart_t* p_uart = (uart_t*) 0x40008000;
```

In Rust, we have some issues:

1. There are no volatile types, only volatile pointer reads/writes. So we
   cannot mark a type as 'volatile' and have all accesses to its fields
   performed a volatile operations. And given that MMIO registers have
   side-effects (like writing to a FIFO), it is important that those
   accesses are volatile.
2. We must never construct a reference to an MMIO peripheral, because
   references are, well, dereferenceable, and LLVM is free to dereference
   them whenever it likes. This might cause unexpected reads of the MMIO
   peripheral and is considered UB.
3. Accessing a field of a struct without constructing a pointer to it used
   to be quite tricky, although as of Rust 1.51 we have
   [`core::ptr::addr_of_mut`] and as of Rust 1.84 we have `&raw mut`.

The usual solution to these problems is to auto-generate code based on some
machine-readable (but non-Rust) description of the MMIO peripheral. This
code will contain functions to get a 'handle' to a peripheral, and that
handle has methods to get a handle to each register within it, and those
handles have methods for reading, writing or modifying the register
contents. Unfortunately, this requires having a machine-readable (typically
SVD XML) description of the peripherals and those are either not always
available, or cover an entire SoC when a driver is in fact only aiming to
work with one common MMIO peripheral (e.g. the Arm PL011 UART that has been
licensed and copy-pasted in dozens of SoC designs).

## How this crate works

This crate presents an alternative solution.

Consider the code:

```rust
#[derive(derive_mmio::Mmio)]
#[repr(C)]
struct Uart {
    data: u32,
    #[mmio(Read)]
    status: u32,
    control: u32,
}
```

Note that your struct must be `repr(C)` and we will check this.

The `derive_mmio::Mmio` derive-macro will generate some new methods and types
for you. You can see this for yourself with `cargo doc` (or `cargo expand` if
you have installed `cargo-expand`), but our example will expand to something
like this (simplified):

```rust
// this is your type, unchanged
#[repr(C)]
struct Uart {
    data: u32,
    status: u32,
    control: u32
}
// this is a new 'handle' type
struct MmioUart {
    ptr: *mut Uart,
}
// some methods on the 'handle' type
impl MmioUart {
    pub fn pointer_to_data(&mut self) -> *mut u32 {
        unsafe { &raw mut (*self.ptr).data }
    }
    pub fn read_data(&self) -> u32 {
        let addr = unsafe { core::ptr::addr_of!((*self.ptr).data) };
        unsafe {
            addr.read_volatile()
        }
    }
    pub fn write_data(&mut self, value: u32) {
        let addr = self.pointer_to_data();
        unsafe { addr.write_volatile(value) }
    }
    pub fn modify_data<F>(&mut self, f: F)
    where
        F: FnOnce(u32) -> u32,
    {
        let value = self.read_data();
        let new_value = f(value);
        self.write_data(new_value);
    }

    // but you can only read the status register
    pub fn pointer_to_status(&mut self) -> *mut u32 {
        unsafe { &raw mut (*self.ptr).status }
    }
    pub fn read_status(&mut self) -> u32 {
        let addr = self.pointer_to_status();
        unsafe { addr.read_volatile() }
    }

    // The control register methods are skipped here for brevity
}
// some new methods we add onto your type
impl Uart {
    pub const unsafe fn new_mmio(ptr: *mut Uart) -> MmioUart {
        MmioUart { ptr }
    }
    pub const unsafe fn new_mmio_at(addr: usize) -> MmioUart {
        MmioUart {
            ptr: addr as *mut Uart,
        }
    }
}
```

OK, that was a lot! Let's unpack it.

## MMIO Wrapper

```rust,ignore
struct MmioUart {
    ptr: *mut Uart,
}
```

This structure, called `Mmio${StructName}` is a handle that proxies access
to that particular peripheral. You create as many as you need by unsafely
calling one of these methods we added to your struct type.

```rust,ignore
impl Uart {
    pub const unsafe fn new_mmio(ptr: *mut Uart) -> MmioUart {
        MmioUart { ptr }
    }
    pub const unsafe fn new_mmio_at(addr: usize) -> MmioUart {
        MmioUart {
            ptr: addr as *mut Uart,
        }
    }
}
```

One is for when you have a pointer, and the other is for when you only have
the address (typically as a literal integer you read from the SoC's
datasheet, like `0x4008_1000`).

It is unsafe to create these - you must verify that you are passing a valid
address or pointer, and that if you are creating multiple MMIO handles for one
peripheral at the same same that you use them in a way that complies with the
peripheral's rules around concurrent access. For example, don't create two
handles and use them to both do a read-modify-write on the *same* register
at the same time - that's a race hazard and the results won't be reliable. But
you could create two and use them to read-modify-write *different* registers -
probably. It depends on whether the registers affect each other or operate
in isolation.

The constructors shown above will be generated by default. You might want to
implement custom constructors, for example if your peripheral is only valid for
one specific address, or a specific set of addresses. You can disable the generation
of these constructors by adding the `#[mmio(no_ctors)]` attribute annotation
to your peripheral block structure.

## MMIO Methods

The MMIO handle has methods to access each of the fields in the underlying
struct.

You can read (which performs a volatile read):

```rust,ignore
println!("data = {}", mmio_uart.read_data());
```

You can write (which performs a volatile write):

```rust,ignore
mmio_uart.write_data(0x00);
```

And you can perform a read-modify-write (which requires exclusive access and
you should not use if any other code might modify this register
concurrently).

```rust,ignore
mmio_uart.modify_control(|mut r| {
    r &= 0xF000_0000;
    r |= 1 << 31;
    r
});
```

If you need a pointer to a register, for example if you want to have a DMA
engine write to a register on your peripheral, you can use this method:

```rust,ignore
let p: *mut u32 = mmio_uart.pointer_to_data();
```

If you have an inner field deriving the [Mmio] type and annotated with `#[mmio(inner)]`,
the derive macro will generate getters for that field. The getter will have the same name
as the field name of your peripheral block and will have a lifetime tied to the outer
MMIO structure. The macro will also generate an `unsafe` `steal_${inner_field}` method
which has a static lifetime, which in turn allows to create an aribtraty number
of owned inner MMIO objects.

For array fields, the macro will generate the following API:

- `pointer_to_${field_name}_start`
- `read_${field_name}` which performs bound checking.
- unsafe `read_${field_name}_unchecked` which does not perform bound checking.
- `write_${field_name}` which performs bound checking.
- unsafe `write_${field_name}_unchecked` which does not perform bound checking.
- `modify_${field_name}` which performs bound checking.
- unsafe `modify_${field_name}_unchecked` which does not perform bound checking.

Except for the pointer method, all APIs expect the array index as the first argument.

## Supported field types

The following field types are supported and tested:

- [u32]
- Arrays of [u32]
- bitfields implemented with [bitbybit::bitfield](https://crates.io/crates/bitbybit)
- Other [Mmio] types which are annotated with the `[mmio(inner)]` attribute.

Other `repr(transparent)` types should work, but you should be careful to ensure
that every field corresponds 1:1 with an MMIO register and that they are the
appropriate size for your CPU architecture.

If you accidentally introduce padding (or, if the sum of the size of the
individual fields isn't the same as the size of the overall `struct`), you will
get a compile error.

## Supported attributes

The following attributes are supported:

### Outer attributes

- `#[mmio(no_ctors)]`: Omit the generation of constructor functions like `new_mmio_at` and
   `new_mmio`. This allows users to specify own custom constructors, for example to constrain
   or check the allowed base addresses.

### Field attributes

The access permission attributes work for array fields as well.

- `#[mmio(PureRead)]`: The field is read-only. The read does not have side effects, and the
   generated reader function only requires a shared reference to the MMIO handle.
- `#[mmio(Read)]`: The field can be read, but the read has side effects. The generated reader
   function requires a mutable reference to the MMIO block.
- `#[mmio(Write)]`: The field can be written to. This will generate a writer function for the
   field.
- `#[mmio(Modify)]`: The field can be modified. This will generate a modify function for the field
   which performs a Read-Modify-Write operation.
- `#[mmio(inner)]`: The field is a register block which also implements [Mmio].
  This will be verified using trait bounds. The derive macro will generate getter functions
  to retrieve the inner block, with the lifetime of the inner block tied to the outer block.

If no permission access modifiers were specified, the library will default to
`PureRead`, `Write`, `Modify` which is the default for most regular R/W registers.
*/

#![no_std]
use core::{fmt::Display, ops::Deref};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OutOfBoundsError(pub usize);

impl Display for OutOfBoundsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "out of bounds access at index {}", self.0)
    }
}

pub struct SharedInnerMmio<T>(T);

impl<T> SharedInnerMmio<T> {
    #[doc(hidden)]
    pub fn __new_internal(t: T) -> Self {
        Self(t)
    }

    pub fn inner(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for SharedInnerMmio<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

#[rustversion::since(1.81)]
impl core::error::Error for OutOfBoundsError {}

/// Marker trait to check whether inner field have implemented Mmio.
///
/// # Safety
///
/// You should not implement this trait yourself. This is done by the [Mmio] derive macro.
pub unsafe trait _MmioMarker {}

/// Const function to check trait bounds.
pub const fn is_mmio<M: _MmioMarker>() {}

#[doc(inline)]
pub use derive_mmio_macro::Mmio;
