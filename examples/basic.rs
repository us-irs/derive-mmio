#[derive(derive_mmio::Mmio)]
#[repr(C)]
struct Uart {
    data: u32,
    control: u32,
}

fn main() {
    println!("Hello, world!");

    let mut uart = Uart {
        data: 0xA,
        control: 0xC,
    };

    // Safety: We're pointing at a real object
    let mut mmio_uart = unsafe { Uart::new_mmio(core::ptr::addr_of_mut!(uart)) };

    println!("data = {}", mmio_uart.read_data());
    mmio_uart.modify_control(|f| {
        println!("control was {f}, is now 32");
        32
    });
    println!("control = {}", mmio_uart.read_control());
}
