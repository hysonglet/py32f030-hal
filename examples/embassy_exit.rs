#![no_std]
#![no_main]

use embedded_hal::digital::InputPin;
use hal::exti::ExtiInput;
use hal::gpio::{PinPullUpDown, PinSpeed};
use hal::mode::Async;
use py32f030_hal as hal;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_time::Timer;

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
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());
    let gpioa = p.GPIOF.split();

    defmt::info!("Example: embassy exti!");

    let key: ExtiInput<_> = ExtiInput::new(gpioa.PF4_BOOT0, PinPullUpDown::No, PinSpeed::Low);
    _spawner.spawn(run(key)).unwrap();

    let mut cnt: u32 = 0;
    loop {
        defmt::info!("high {} ", cnt);
        cnt += 1;
        Timer::after_secs(5).await;
    }
}
