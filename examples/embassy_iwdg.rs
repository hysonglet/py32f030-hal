#![no_std]
#![no_main]

use py32f030_hal as hal;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::iwdg::{Config, IWdg};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());
    defmt::info!("Hello World!");

    let config: Config = Default::default();
    defmt::info!("iwdg timeout: {}us", config.timeout_us());
    let iwdg = IWdg::new(p.IWdg, config);
    iwdg.start();

    let mut cnt: u32 = 0;
    loop {
        defmt::info!("time {} ", cnt);
        iwdg.feed();
        cnt += 1;
        // 10 秒内喂狗
        if cnt <= 10 {
            Timer::after_millis(1000).await;
        } else {
            // 10秒后等待喂狗超时
            Timer::after_secs(30).await;
        }
    }
}
