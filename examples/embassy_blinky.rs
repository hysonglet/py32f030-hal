//! 在不同的任务中闪烁不同的 LED， 学习多任务操作
//!

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_hal::digital::StatefulOutputPin;
use hal::gpio::{Output, PinIoType, PinSpeed};
use py32f030_hal::{
    self as hal,
    gpio::{AnyPin, Pin},
};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task(pool_size = 2)]
async fn run_led(led: AnyPin, delay_ms: u64) {
    let mut led = Output::new(led, PinIoType::PullDown, PinSpeed::Low);
    loop {
        let _ = led.toggle();
        Timer::after_millis(delay_ms).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = hal::init(Default::default());

    defmt::info!("Testing the flashing of different LEDs in multi-tasking.");

    let gpioa = p.GPIOA.split();

    // spawner.must_spawn(run_led(gpioa.PA9.degrade(), 1000));
    // spawner.must_spawn(run_led(gpioa.PA10.degrade(), 2000));

    spawner.spawn(run_led(gpioa.PA9.degrade(), 1000)).unwrap();
    spawner.spawn(run_led(gpioa.PA10.degrade(), 500)).unwrap();

    loop {
        Timer::after_secs(2).await;
    }
}
