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

    // 引脚指定方式3
    // PA0，指定为OC_N, 当为None时候不会被使用，当引脚没有被模版冲突则不需要指定模版类型
    pwm.set_channel_1_pin(Some(gpioa.PA2));

    // 配置定时器
    pwm.config(
        /* 配置通道1 */
        Some(ChannelConfig::default()),
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
        cortex_m::asm::wfi();
    }
}
