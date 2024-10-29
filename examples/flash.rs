#![no_std]
#![no_main]

use hal::flash::Flash;
use py32f030_hal as hal;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("examples: key");
    let p = hal::init(Default::default());
    let flash = Flash::new(p.FLASH);

    let uuid = Flash::<hal::mcu::peripherals::FLASH>::uuid();

    defmt::info!("uuid: {:x}", uuid);
    loop {}
}
