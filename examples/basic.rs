//! A basic example of using the Mmio trait.
//!
//! We use a 'fake' UART so this doesn't need any specific hardware to run.

#[derive(derive_mmio::Mmio)]
#[repr(C)]
struct Uart {
    data: u32,
    control: u32,
    _reserved: u32,
}

fn main() {
    let mut uart = Uart {
        data: 0xA,
        control: 0xC,
        _reserved: 0,
    };

    // Safety: We're pointing at a real object
    let mut mmio_uart = unsafe { Uart::new_mmio(core::ptr::addr_of_mut!(uart)) };
    println!("sample UART is @ {:p}", core::ptr::addr_of_mut!(uart));

    println!("data = {}", mmio_uart.read_data());
    println!("data register is at = {:p}", mmio_uart.pointer_to_data());
    mmio_uart.modify_control(|f| {
        println!("control was {f}, is now 32");
        32
    });
    println!("control = {}", mmio_uart.read_control());
    println!("control register is @ {:p}", mmio_uart.pointer_to_control());
}
