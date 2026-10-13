#![no_std]
#![no_main]

use defmt::Debug2Format;
use hal::flash::Flash;
use hal::mcu::peripherals::FLASH;
use py32f030_hal as hal;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("examples: key");
    let p = hal::init(Default::default());
    let flash = Flash::new(p.FLASH);

    let uuid = Flash::<FLASH>::uuid();

    defmt::info!("uuid: {:x}", uuid);

    let addr = Flash::<FLASH>::get_page_addr(15, 0).unwrap();

    // Erase sector 15, page 0.
    let rst = flash.erase_page_by_index(15, 0);
    defmt::info!("erase_page_by_index: rst: {}", Debug2Format(&rst));

    let mut page_buf: [u32; 32] = [0x12345678; 32];

    // Program sector 15, page 0.
    let rst = flash.program_page(addr, page_buf);
    defmt::info!("program_page rst: {}", Debug2Format(&rst));

    // Read sector 15, page 0.
    let rst = flash.read_page_by_index(15, 0, &mut page_buf);

    defmt::info!(
        "read_page_by_index: rst: {} {:x}",
        Debug2Format(&rst),
        page_buf
    );

    let d: u32 = unsafe { core::ptr::read_volatile(addr as *const u32) };
    defmt::info!("{:x} {:x}", addr, d);

    loop {
        cortex_m::asm::wfe();
    }
}
