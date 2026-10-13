#![no_std]
#![no_main]

// use hal::gpio::{Af, PinIoType, PinSpeed};
// use hal::timer::advanced_timer::TimerChannel1Pin;
use hal::timer::advanced_timer::{AnyTimer, ChannelConfig, ChannelOutputConfig};
use py32f030_hal::gpio::gpioa::PA0;
use py32f030_hal::{self as hal, mode::Blocking, timer::advanced_timer::Channel};

use embassy_executor::Spawner;
use embassy_time::Timer;
// use hal::mcu::peripherals::TIM1;
use embedded_hal_027::Pwm;

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
    let mut pwm = timer.as_pwm();

    let channel_1_pin = gpioa.PA3;

    // Pin assignment method 1.
    // let _oc1_pin = Af::new(
    //     channel_1_pin, //gpioa.PA3,
    //     PinAF::AF13,
    //     PinSpeed::VeryHigh,
    //     PinIoType::PullUp,
    // );

    // Pin assignment method 2.
    // channel_1_pin.set_instance_af(PinSpeed::VeryHigh, PinIoType::PullUp);

    // Pin assignment method 3.
    // PA0 assigned as OC_N; unused when None, and no template type is needed if the pin has no template conflict.
    pwm.set_channel_1_pins::<_, PA0>(Some(channel_1_pin), None);

    // Configure the timer.
    pwm.config(
        /* configure channel 1 */
        Some(ChannelConfig::default().ch(ChannelOutputConfig::default())),
        None,
        None,
        None,
    );

    // Counting frequency is 1 MHz.
    pwm.set_frequency(1_000_000);
    // Set the counting period to 1000, so the waveform frequency is 1000_000/1000 = 1K.
    pwm.set_period(1000u16 - 1);
    let max_duty = pwm.get_max_duty();
    // 33% duty cycle.
    pwm.set_duty(Channel::CH1, max_duty / 3);
    // Enable the channel.
    pwm.enable(Channel::CH1);
    // Start the counter.
    pwm.start();

    loop {
        Timer::after_secs(1).await;
    }
}
