/*!
# 'derive-mmio' - turning structures into MMIO access objects

In C it is very common to create structures that refer to MMIO peripherals:

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
   references are, well, dereferencable, and LLVM is free to derefererence
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

This crate presents an alternative solution.

Consider the code:

```rust
#[derive(derive_mmio::Mmio)]
#[repr(C)]
struct Uart {
    data: u32,
    status: u32,
}
```

Note that your struct must be `repr(C)` and we will check this.

The `derive_mmio::Mmio` derive-macro will generate some new methods and types
for you. You can see this for yourself with `cargo doc` (or `cargo expand` if
you have installed `cargo-expand`), but our example will expand to something
like:

```rust
// this is your type, unchanged
#[repr(C)]
struct Uart {
    data: u32,
    status: u32,
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
    pub fn read_data(&mut self) -> u32 {
        let addr = self.pointer_to_data();
        unsafe { addr.read_volatile() }
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

    // and the same again for the 'control' register
}
// some new methods we add onto your type
impl Uart {
    pub unsafe fn new_mmio(ptr: *mut Uart) -> MmioUart {
        MmioUart { ptr }
    }
    pub unsafe fn new_mmio_at(addr: usize) -> MmioUart {
        MmioUart {
            ptr: addr as *mut Uart,
        }
    }
}
```

OK, that was a lot! Let's unpack it.

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
    pub unsafe fn new_mmio(ptr: *mut Uart) -> MmioUart {
        MmioUart { ptr }
    }
    pub unsafe fn new_mmio_at(addr: usize) -> MmioUart {
        MmioUart {
            ptr: core::ptr::with_exposed_provenance_mut(addr),
        }
    }
}
```

One is for when you have a pointer, and the other is for when you only have
the address (typically as a literal integer you read from the SoC's
datasheet, like `0x4008_1000`).

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

In the event you want to have a DMA engine write to a register on your
peripheral, you can use this method:

```rust,ignore
let p = mmio_uart.pointer_to_data();
```
*/

#![no_std]

#[doc(inline)]
pub use derive_mmio_macro::Mmio;
