//! A basic example of using the the inner MMIO attribute.
//!
//! You can expand this example by running
//!
//! ```rs
//! cargo expand --example inner_block
//! ```

mod inner {
    #[derive(derive_mmio::Mmio)]
    #[repr(C)]
    pub struct UartBank {
        data: u32,
        status: u32,
    }

    impl UartBank {
        pub fn fake() -> UartBank {
            UartBank {
                data: 0x2,
                status: 0x3,
            }
        }
    }
}

#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct Uart {
    control: u32,
    #[mmio(inner)]
    bank_0: inner::UartBank,
    #[mmio(inner)]
    bank_1: inner::UartBank,
}

fn main() {
    let mut uart = Uart {
        control: 0xC,
        bank_0: inner::UartBank::fake(),
        bank_1: inner::UartBank::fake(),
    };

    // Safety: We're pointing at a real object
    let mut mmio_uart = unsafe { Uart::new_mmio(core::ptr::addr_of_mut!(uart)) };
    let mut bank0 = mmio_uart.bank_0();
    let bank0_data = bank0.read_data();
    assert_eq!(bank0_data, 0x2);
    let bank0_status = bank0.read_status();
    assert_eq!(bank0_status, 0x3);
    bank0.write_data(0x42);
    assert_eq!(bank0.read_data(), 0x42);

    // You can't do this, because the bank0 value is holding
    // a mutable borrow on the underlying UART. This prevents you creating
    // two handles to the same block (but also to any other field).
    // let foo = mmio_uart.read_control();

    bank0.write_status(0x5);
    assert_eq!(bank0.read_status(), 0x5);

    let bank1_data = mmio_uart.bank_1().read_data();
    assert_eq!(bank1_data, 0x2);
    let bank1_data = mmio_uart.bank_1().read_status();
    assert_eq!(bank1_data, 0x3);

    // Can only use shared API here.
    let bank0_shared = mmio_uart.bank_0_shared();
    assert_eq!(bank0_shared.read_data(), 0x42);
}
