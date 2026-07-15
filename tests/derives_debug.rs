#[derive(derive_mmio::Mmio)]
#[repr(C)]
struct Uart {
    // this is read-write by default
    data: u32,
}

pub fn main() {
    let mut uart = Uart { data: 0xA };

    // Safety: We're pointing at a real object
    let mmio_uart = unsafe { Uart::new_mmio(core::ptr::addr_of_mut!(uart)) };
    let addr = core::ptr::addr_of!(uart);
    // Verify that the debug implementation simply shows the base address.
    assert_eq!(
        format!("{:?}", mmio_uart),
        format!("MmioUart {{ ptr: {:?} }}", addr)
    );
}
