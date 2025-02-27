#[derive(derive_mmio::Mmio)]
#[mmio(no_ctors_x)]
#[repr(C)]
struct Uart {
    data: u32,
}

fn main() {
    // Can not really test this on a host environment. Simply verify that it builds..
}
