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

    // 引脚指定方式1
    // let _oc1_pin = Af::new(
    //     channel_1_pin, //gpioa.PA3,
    //     PinAF::AF13,
    //     PinSpeed::VeryHigh,
    //     PinIoType::PullUp,
    // );

    // 引脚指定方式2
    // channel_1_pin.set_instance_af(PinSpeed::VeryHigh, PinIoType::PullUp);

    // 引脚指定方式3
    // PA0，指定为OC_N, 当为None时候不会被使用，当引脚没有被模版冲突则不需要指定模版类型
    pwm.set_channel_1_pin::<_, PA0>(Some(channel_1_pin), None);

    // 配置定时器
    pwm.config(
        /* 配置通道1 */
        Some(ChannelConfig::default().ch(ChannelOutputConfig::default())),
        None,
        None,
        None,
    );

    // 计数频率为1M
    pwm.set_frequency(1_000_000);
    // 设置计数周期为1000，则波形的频率为 1000_000/1000 = 1K
    pwm.set_period(1000u16 - 1);
    let max_duty = pwm.get_max_duty();
    // 33%的占空比
    pwm.set_duty(Channel::CH1, max_duty / 3);
    // 使能通道
    pwm.enable(Channel::CH1);
    // 开始计数器
    pwm.start();

    loop {
        Timer::after_secs(1).await;
    }
}
