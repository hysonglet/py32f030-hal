#![no_std]
#![no_main]

use embedded_hal::digital::v2::ToggleableOutputPin;
use hal::gpio::{Output, PinIoType, PinSpeed};
use py32f030_hal as hal;

use panic_halt as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());
    let gpioa = p.GPIOA.split();
    let mut led = Output::new(gpioa.PA0, PinIoType::PullUp, PinSpeed::Low);

    loop {
        let _ = led.toggle();
        cortex_m::asm::delay(10000000);
    }
}
