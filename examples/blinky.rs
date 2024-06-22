#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

// use core::mem::{size_of, size_of_val};

use py32f030_hal as hal;
use hal::{InputPin, OutputPin, hal::digital::v2::ToggleableOutputPin};

const LOOP_CNT: u32 = 200000;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");
    defmt::info!("info");
    defmt::trace!("trace");
    defmt::warn!("warn");
    defmt::debug!("debug");
    defmt::error!("error");

    let port = hal::gpio::GPIOA::Port;
    let p = port.split();

    let mut led = p.PA11.into_output();

    led.set_output_type(hal::gpio::PinOutputType::PushPull).unwrap();
    let _ = led.set_low().unwrap();
    let _ = led.set_high().unwrap();

    defmt::info!("led: {}", led.is_high().unwrap());
    
    loop {
        for _ in 0..LOOP_CNT {
            cortex_m::asm::nop();
        }
        led.toggle().unwrap();
    }
}
