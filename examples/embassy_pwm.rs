#![no_std]
#![no_main]

use hal::{mode::Async, timer::advanced_timer::AnyTimer};
use py32f030_hal::{self as hal, gpio, mode::Blocking, timer::advanced_timer::Channel};

use embassy_executor::Spawner;
use embassy_time::Timer;
// use hal::mcu::peripherals::TIM1;
use embedded_hal::Pwm;
use hal::timer::advanced_timer::TimerChannel1Pin;

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
    let gpioa = p.GPIOA.split();

    _spawner.spawn(run()).unwrap();

    let timer: AnyTimer<_, Blocking> = AnyTimer::new(p.TIM1).unwrap();
    let channel_1_pin = gpioa.PA8;
    channel_1_pin.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::PullUp);
    let mut pwm = timer.as_pwm();
    pwm.set_frequency(1000);
    pwm.set_period(3000u16);
    let max_duty = pwm.get_max_duty();
    defmt::info!("max duty: {}", max_duty);
    pwm.set_duty(Channel::CH1, max_duty / 2);

    pwm.enable(Channel::CH1);
    pwm.start();

    let mut cnt = 0;
    loop {
        // defmt::info!("{}", cnt);
        pwm.debug();
        Timer::after_secs(5).await;
        cnt += 1;
    }
}
