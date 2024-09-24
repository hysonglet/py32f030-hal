#![no_std]
#![no_main]

use embedded_hal_async::delay::DelayNs;

use hal::{mode::Async, timer::advanced_timer::AnyTimer};
use py32f030_hal as hal;

use embassy_executor::Spawner;
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn run() {
    loop {
        Timer::after_secs(2).await;
        defmt::info!("task run");
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("time1 start...");
    let p = hal::init(Default::default());

    _spawner.spawn(run()).unwrap();

    let timer: AnyTimer<_, Async> = AnyTimer::new(p.TIM1).unwrap();
    let mut counter = timer.as_counter();

    let mut cnt = 0;
    loop {
        defmt::info!("{}", cnt);
        counter.delay_ms(1000).await;
        // counter.delay_us(1000_000).await;
        cnt += 1;
    }
}
