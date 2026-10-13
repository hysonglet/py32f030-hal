#![no_std]
#![no_main]

use hal::timer::general_purpose_timer::{AnyTimer, Channel, ChannelConfig};
use py32f030_hal::{self as hal, mode::Blocking};

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("timer pwm examples start...");
    let p = hal::init(Default::default());
    let gpioa = p.GPIOA.split();

    let timer: AnyTimer<_, Blocking> = AnyTimer::new(p.TIM3).unwrap();
    let mut pwm = timer.as_pwm();

    // Pin assignment method 3.
    // PA0 assigned as OC_N; unused when None, and no template type is needed if the pin has no template conflict.
    pwm.set_channel_1_pin(Some(gpioa.PA2));

    // Configure the timer.
    pwm.config(
        /* configure channel 1 */
        Some(ChannelConfig::default()),
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
        cortex_m::asm::wfi();
    }
}
