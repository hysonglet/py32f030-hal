#![no_std]
#![no_main]

use embedded_hal::digital::v2::ToggleableOutputPin;
use embedded_hal::timer::CountDown;
use fugit::{Duration, ExtU32, ExtU64, Rate, RateExtU32};
use fugit::{HertzU32, MegahertzU32, MicrosDurationU32, TimerDurationU32, TimerInstantU32};
// use fugit_timer::Timer;
use hal::mode::Blocking;
use hal::timer::advanced_timer::AnyTimer;
use py32f030_hal as hal;

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("timer counter examples start...");
    let p = hal::init(Default::default());

    let timer = AnyTimer::<_, Blocking>::new(p.TIM1).unwrap();
    let mut counter = timer.as_counter();

    // // Efficient short-hands (`.millis()`, ...)
    // let d = Duration::<u32, 1, 1_000>::from_ticks(2000);
    // defmt::info!("{} {} {}", d.to_secs(), d.ticks(), d.to_nanos());
    // let rate: HertzU32 = d.into_rate();
    // defmt::info!("{:?}", rate.raw());

    loop {
        // 延时 1s
        // counter.delay_us_blocking(1000_000);
        let _ = counter.start(2u64.millis());
        let _ = counter.start(2u64.secs());
        let _ = counter.start(2u64.minutes());
        let _ = counter.start(2u64.hours());
        loop {}
    }
}
