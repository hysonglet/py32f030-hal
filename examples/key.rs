#![no_std]
#![no_main]

use {defmt::*, defmt_rtt as _, panic_probe as _};

use hal::delay;
use hal::gpio::{Input, Pull, Speed};
use py32f030_hal::{self as hal, prelude::*};

#[cortex_m_rt::entry]
fn main() -> ! {
    println!("examples: key");

    let p = hal::init(Default::default());

    let gpioa = p.GPIOF.split();

    let mut key = Input::new(gpioa.PF4_BOOT0, Pull::None, Speed::Low);

    loop {
        info!("key: {}", key.is_low());
        delay::delay_ms(1000);
    }
}
