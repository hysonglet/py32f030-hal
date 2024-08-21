#![no_std]
#![no_main]

use py32f030_hal as _;

// use panic_halt as _;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("hello world");
    loop {}
}
