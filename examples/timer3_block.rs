#![no_std]
#![no_main]

use embedded_hal::timer::CountDown;
use fugit::ExtU32;
use hal::mode::Blocking;
use hal::timer::general_purpose_timer::AnyTimer;
use py32f030_hal as hal;

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("timer counter examples start...");
    let p = hal::init(Default::default());

    let timer = AnyTimer::<_, Blocking>::new(p.TIM3).unwrap();
    let mut counter = timer.as_counter();

    let mut cnt = 0;
    loop {
        // 延时 5s
        defmt::info!("repeat...{} ", cnt);
        counter.start(60 * 5u32.secs());
        let _ = counter.wait();
        cnt += 1;
    }
}
