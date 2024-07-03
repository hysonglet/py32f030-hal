#![no_std]
#![no_main]

use py32f030_hal as hal;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_time::Timer;

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
async fn main(_spawner: Spawner) {
    let _p = hal::init(Default::default());
    defmt::info!("Hello World!");

    _spawner.spawn(run()).unwrap();

    let mut cnt: u32 = 0;
    loop {
        defmt::info!("high {} ", cnt);
        cnt += 1;
        Timer::after_secs(5).await;
    }
}
