#![no_std]
#![no_main]

use py32f030_hal::mode::Async;
use py32f030_hal::{self as hal};

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::rtc::AnyRtc;

// use panic_halt as _;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn run() {
    let mut cnt: u32 = 0;
    loop {
        defmt::info!("task run {} ", cnt);
        cnt += 2;
        Timer::after_secs(2).await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());

    let rtc: AnyRtc<_, Async> = AnyRtc::new(p.RTC, Default::default()).unwrap();

    defmt::info!("start: {}", rtc.read());
    _spawner.spawn(run()).unwrap();

    loop {
        rtc.wait_second(3).await;

        defmt::info!("rtc: {}", rtc.read());
    }
}
