#![no_std]
#![no_main]

use hal::gpio::{Output, PinIoType, Speed};
use hal::mode::Blocking;
use hal::timer::advanced_timer::AnyTimer;
use py32f030_hal::{self as hal, prelude::*};

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("timer counter examples start...");
    let p = hal::init(Default::default());

    let gpioa = p.GPIOA.split();

    let timer = AnyTimer::<_, Blocking>::new(p.TIM1).unwrap();
    let mut counter = timer.as_counter();

    let mut led = Output::new(gpioa.PA0, PinIoType::PullUp, Speed::Low);

    let mut cnt = 0;

    loop {
        let _ = led.toggle();
        // 延时 1s
        counter.delay_us_blocking(1_000_000);
        defmt::info!("{}", cnt);
        cnt += 1;
    }
}
