//! Experimentation test module.
#![allow(dead_code)]

//#[derive(derive_mmio::Mmio)]
#[repr(C)]
struct UartBank {
    // this is read-write by default
    data: u32,
    status: u32,
}

pub struct MmioUartBank<'a> {
    ptr: *mut UartBank,
    phantom: core::marker::PhantomData<&'a mut ()>,
}

impl MmioUartBank<'_> {
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

    pub fn pointer_to_status(&mut self) -> *mut u32 {
        unsafe { &raw mut (*self.ptr).status }
    }
    pub fn read_status(&mut self) -> u32 {
        let addr = self.pointer_to_status();
        unsafe { addr.read_volatile() }
    }
    pub fn write_status(&mut self, value: u32) {
        let addr = self.pointer_to_status();
        unsafe { addr.write_volatile(value) }
    }
    pub fn modify_status<F>(&mut self, f: F)
    where
        F: FnOnce(u32) -> u32,
    {
        let value = self.read_data();
        let new_value = f(value);
        self.write_data(new_value);
    }
}
//impl<'a> MmioUartBank<'a> {}

//#[derive(derive_mmio::Mmio)]
#[repr(C)]
struct Uart {
    // you can be explicit if you like
    control: u32,
    //#[mmio(inner)]
    bank_0: UartBank,
    //#[mmio(inner)]
    bank_1: UartBank,
}

// this is a new 'handle' type
struct MmioUart<'a> {
    ptr: *mut Uart,
    phantom: core::marker::PhantomData<&'a mut ()>,
}

// some methods on the 'handle' type
impl MmioUart<'_> {
    pub fn pointer_to_control(&mut self) -> *mut u32 {
        unsafe { &raw mut (*self.ptr).control }
    }
    pub fn read_control(&mut self) -> u32 {
        let addr = self.pointer_to_control();
        unsafe { addr.read_volatile() }
    }
    pub fn write_control(&mut self, value: u32) {
        let addr = self.pointer_to_control();
        unsafe { addr.write_volatile(value) }
    }
    pub fn modify_control<F>(&mut self, f: F)
    where
        F: FnOnce(u32) -> u32,
    {
        let value = self.read_control();
        let new_value = f(value);
        self.write_control(new_value);
    }
    pub fn bank_0(&mut self) -> MmioUartBank<'_> {
        MmioUartBank {
            ptr: unsafe { &mut (*self.ptr).bank_0 },
            phantom: std::marker::PhantomData,
        }
    }
    pub fn bank_1(&mut self) -> MmioUartBank<'_> {
        MmioUartBank {
            ptr: unsafe { &mut (*self.ptr).bank_1 },
            phantom: std::marker::PhantomData,
        }
    }
}

// some new methods we add onto your type
impl Uart {
    pub const unsafe fn new_mmio(ptr: *mut Uart) -> MmioUart<'static> {
        MmioUart {
            ptr,
            phantom: core::marker::PhantomData,
        }
    }
    pub const unsafe fn new_mmio_at(addr: usize) -> MmioUart<'static> {
        Self::new_mmio(addr as *mut Uart)
    }
}

fn main() {
    let mut uart = Uart {
        control: 0xC,
        bank_0: UartBank {
            data: 0x1,
            status: 0x2,
        },
        bank_1: UartBank {
            data: 0x3,
            status: 0x4,
        },
    };

    // Safety: We're pointing at a real object
    let mut mmio_uart = unsafe { Uart::new_mmio(core::ptr::addr_of_mut!(uart)) };
    let mut bank0 = mmio_uart.bank_0();
    let bank0_data = bank0.read_data();
    assert_eq!(bank0_data, 0x1);
    let bank0_status = bank0.read_status();
    assert_eq!(bank0_status, 0x2);
    bank0.write_status(0x5);
    assert_eq!(bank0.read_status(), 0x5);
    let bank1_data = mmio_uart.bank_1().read_data();
    assert_eq!(bank1_data, 0x3);
    let bank1_data = mmio_uart.bank_1().read_status();
    assert_eq!(bank1_data, 0x4);
}
