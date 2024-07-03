#![no_std]
#![no_main]

use embedded_hal::digital::v2::InputPin;
use hal::exit::ExtiInput;
use hal::gpio::{PinPullUpDown, PinSpeed};
use py32f030_hal as hal;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_time::Timer;

#[embassy_executor::task]
async fn run(key: ExtiInput<'static>) {
    loop {
        defmt::info!("wating for key push...");
        key.wait_for_low().await;
        defmt::info!("key pushed {}", key.is_high());
        key.wait_for_high().await;
        defmt::info!("key released");
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());
    let gpioa = p.GPIOA.split();

    defmt::info!("Hello World!");

    let key: ExtiInput = ExtiInput::new(gpioa.PA12, PinPullUpDown::No, PinSpeed::Low);
    _spawner.spawn(run(key)).unwrap();

    let mut cnt: u32 = 0;
    loop {
        defmt::info!("high {} ", cnt);
        cnt += 1;
        Timer::after_secs(5).await;
    }
}
