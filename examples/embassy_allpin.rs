//! 在不同的任务中闪烁不同的 LED， 学习多任务操作
//!

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;
use embedded_hal_027::digital::v2::ToggleableOutputPin;
use hal::gpio::{Output, PinIoType, PinSpeed};
use py32f030_hal::{
    self as hal,
    gpio::{AnyPin, Pin},
};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task(pool_size = 26)]
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
    let gpiob = p.GPIOB.split();
    let gpiof = p.GPIOF.split();

    let pins = [
        gpioa.PA0.degrade(),
        gpioa.PA1.degrade(),
        gpioa.PA2.degrade(),
        gpioa.PA3.degrade(),
        gpioa.PA4.degrade(),
        gpioa.PA5.degrade(),
        gpioa.PA6.degrade(),
        gpioa.PA7.degrade(),
        gpioa.PA8.degrade(),
        gpioa.PA9.degrade(),
        gpioa.PA10.degrade(),
        gpioa.PA11.degrade(),
        gpioa.PA12.degrade(),
        // gpioa.PA13.degrade(),
        // gpioa.PA14.degrade(),
        gpioa.PA15.degrade(),
        gpiob.PB0.degrade(),
        gpiob.PB1.degrade(),
        gpiob.PB2.degrade(),
        gpiob.PB3.degrade(),
        gpiob.PB4.degrade(),
        gpiob.PB5.degrade(),
        gpiob.PB6.degrade(),
        gpiob.PB7.degrade(),
        gpiob.PB8.degrade(),
        gpiof.PF3.degrade(),
    ];

    for pin in pins {
        spawner.spawn(run_led(pin, 100)).unwrap();
    }

    loop {
        Timer::after_secs(2).await;
    }
}
