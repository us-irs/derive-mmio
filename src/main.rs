#[derive(mmio::Mmio)]
struct Uart {
    data: u32,
    status: u32,
    control: u32,
}

fn main() {
    println!("Hello, world!");

    let mut uart = Uart {
        data: 0xA,
        status: 0xB,
        control: 0xC,
    };

    let mut wrapper = Uart::mmio(&raw mut uart);

    println!("data = {}", wrapper.read_data());
    wrapper.modify_control(|f| {
        println!("control was {f}, is now 32");
        32
    });
    println!("control = {}", wrapper.read_control());
}
