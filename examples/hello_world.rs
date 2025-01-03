#![no_std]
#![no_main]

use py32f030_hal as _;
use {defmt::info, defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    info!("hello world");
    loop {
        cortex_m::asm::wfe();
    }
}
