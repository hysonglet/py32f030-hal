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

    let mut cnt = 0;
    loop {
        // 延时 5s
        defmt::info!("repeat...{} ", cnt);
        let _ = counter.start(30u32.secs());
        let _ = counter.wait();
        cnt += 1;
    }
}
