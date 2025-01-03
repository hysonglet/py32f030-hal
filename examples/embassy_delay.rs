#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use py32f030_hal as hal;
use {defmt::info, defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn run() {
    let mut cnt: u32 = 0;
    loop {
        defmt::info!("low {} ", cnt);
        cnt += 2;
        Timer::after_secs(2).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = hal::init(Default::default());
    info!("Hello World!");

    spawner.spawn(run()).unwrap();

    let mut cnt: u32 = 0;
    loop {
        info!("high {} ", cnt);
        cnt += 1;
        Timer::after_secs(5).await;
    }
}
