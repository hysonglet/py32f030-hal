#![no_std]
#![no_main]

use hal::gpio::{Output, PinIoType, Speed};
use py32f030_hal::{self as hal, prelude::*};
use {defmt::info, defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());

    info!("Led blinky testing...");

    let gpioa = p.GPIOA.split();
    // LED: RX led
    let mut led = Output::new(gpioa.PA10, PinIoType::PullDown, Speed::Low);

    loop {
        // 翻转led
        let _ = led.toggle();
        cortex_m::asm::delay(10_000_000);
    }
}
