#![no_std]
#![no_main]

use embedded_hal::digital::v2::ToggleableOutputPin;
use hal::gpio::{Output, PinIoType, PinSpeed};
use py32f030_hal as hal;

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());
    let gpioa = p.GPIOA.split();
    let mut led = Output::new(gpioa.PA10, PinIoType::PullDown, PinSpeed::Low);

    loop {
        // 翻转led
        let _ = led.toggle();
        cortex_m::asm::delay(10_000_000);
        defmt::info!("XXXX");
    }
}
