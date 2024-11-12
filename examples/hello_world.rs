#![no_std]
#![no_main]

use py32f030_hal as _;

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main_fun() -> ! {
    defmt::info!("hello world");
    loop {
        cortex_m::asm::wfe();
    }
}
