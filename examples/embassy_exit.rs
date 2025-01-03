#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::exti::ExtiInput;
use hal::gpio::{Pull, Speed};
use hal::mode::Async;
use py32f030_hal::{self as hal, prelude::*};
use {defmt::info, defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn run(mut key: ExtiInput<'static, Async>) {
    loop {
        defmt::info!("wating for key push...");
        key.wait_for_low().await;
        defmt::info!("key pushed {}, and wating for key release", key.is_high());
        key.wait_for_high().await;
        defmt::info!("key released");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = hal::init(Default::default());
    let gpioa = p.GPIOF.split();

    info!("Example: embassy exti!");

    let key: ExtiInput<_> = ExtiInput::new(gpioa.PF4_BOOT0, Pull::None, Speed::Low);
    spawner.spawn(run(key)).unwrap();

    let mut cnt: u32 = 0;
    loop {
        info!("high {} ", cnt);
        cnt += 1;
        Timer::after_secs(5).await;
    }
}
