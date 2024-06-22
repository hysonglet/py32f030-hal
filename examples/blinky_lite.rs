#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use py32f030_hal as hal;
use hal::{InputPin, OutputPin, hal::digital::v2::ToggleableOutputPin};

const LOOP_CNT: u32 = 200000;

#[cortex_m_rt::entry]
fn main() -> ! {

    let port = hal::gpio::GPIOA::Port;
    let p = port.split();

    let mut led = p.PA11.into_output();

    led.set_output_type(hal::gpio::PinOutputType::PushPull).unwrap();
    let _ = led.set_low().unwrap();
    let _ = led.set_high().unwrap();

    loop {
        for _ in 0..LOOP_CNT {
            cortex_m::asm::nop();
        }
        led.toggle().unwrap();
    }
}
