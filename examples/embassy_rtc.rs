#![no_std]
#![no_main]

use embassy_futures::select;
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
async fn main(spawner: Spawner) {
    let p = hal::init(Default::default());

    let rtc: AnyRtc<_, Async> = AnyRtc::new(p.RTC, Default::default()).unwrap();

    defmt::info!("start: {}", rtc.read());
    spawner.spawn(run()).unwrap();

    loop {
        rtc.wait_alarm(3).await;
        defmt::info!("rtc: {}", rtc.read());

        rtc.wait_second().await;
        defmt::info!("rtc: {}", rtc.read());

        select::select(rtc.wait_alarm(3), rtc.wait_second()).await;
        defmt::info!("rtc: {}", rtc.read());
    }
}
