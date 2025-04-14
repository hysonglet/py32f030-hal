#![no_std]
#![no_main]

use core::u16;

use hal::gpio::{Af, PinIoType, Speed};
// use hal::timer::advanced_timer::TimerChannel1Pin;
use hal::timer::advanced_timer::{AnyTimer, ChannelConfig, ChannelOutputConfig};
use py32f030_hal::gpio::gpioa::PA0;
use py32f030_hal::gpio::PinAF;
use py32f030_hal::{self as hal, mode::Blocking, timer::advanced_timer::Channel};

use embassy_executor::Spawner;
use embassy_time::Timer;
// use hal::mcu::peripherals::TIM1;
use embedded_hal_027::Pwm;

use defmt::info;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("time1 start...");
    let p = hal::init(Default::default());
    let gpioa = p.GPIOA.split();

    let timer: AnyTimer<_, Blocking> = AnyTimer::new(p.TIM1).unwrap();
    let mut pwm = timer.as_pwm();

    let channel_1_pin = gpioa.PA3;

    // // 引脚指定方式1
    // let _oc1_pin = Af::new(
    //     channel_1_pin, //gpioa.PA3,
    //     PinAF::AF13,
    //     Speed::VeryHigh,
    //     PinIoType::PullUp,
    // );

    // 引脚指定方式2
    // channel_1_pin.set_instance_af(PinSpeed::VeryHigh, PinIoType::PullUp);

    // 引脚指定方式3
    // PA0，指定为OC_N, 当为None时候不会被使用，当引脚没有被模版冲突则不需要指定模版类型
    pwm.set_channel_1_pin::<_, PA0>(Some(channel_1_pin), Some(gpioa.PA0));

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
    pwm.set_duty(Channel::CH1, 50);
    // 设置计数周期为1000，则波形的频率为 1000_000/1000 = 1K
    // pwm.set_period(1000u16 - 1);
    // let max_duty = pwm.get_max_duty();
    // // 33%的占空比
    // pwm.set_duty(Channel::CH1, max_duty / 3);
    // 使能通道
    pwm.enable(Channel::CH1);
    // 开始计数器
    pwm.start();

    loop {
        for note in &MIDI_CONTENT {
            let delay = note.delay;
            let channel = note.channel;
            let note = note.note as u32;
            // 只播放指定的 通道
            if channel == 0 {
                let period = (1000_000.0 / NOTE_FREQ[note as usize] - 1.0) as u16;

                info!("freq: {}, note: {}, delay: {}", period, note, delay);

                Timer::after_millis((delay) as u64).await;
                pwm.set_period(period);
            }
        }
    }
}

const NOTE_FREQ: [f32; 128] = [
    // 8.18, /* 0 */
    0.05, /* 0 */
    8.66, 9.18, 9.72, 10.3, 10.91, 11.56, 12.25, 12.98, 13.75, 14.57, /* 1~10 */
    15.43, 16.35, 17.32, 18.35, 19.45, 20.6, 21.83, 23.12, 24.5, 25.96, 27.5, /* 11~21 */
    29.14, 30.87, 32.7, 34.65, 36.71, 38.89, 41.2, 43.65, 46.25, 49.0, 51.91, /* 22~32 */
    55.0, 58.27, 61.74, 65.41, 69.3, 73.42, 77.78, 82.41, 87.31, 92.5, 48.99, /* 33~43 */
    51.91, 55.00, 58.27, 61.74, 65.41, 69.30, 73.42, 77.78, 82.41, 87.31, 92.5, /* 44~54 */
    98.0, 103.8, 110.0, 116.5, 123.5, 130.8, 138.6, 146.8, 155.6, 164.8, 174.6, /* 55~65 */
    185.0, 196.0, 207.7, 220.0, 233.1, 246.9, 261.6, 277.2, 293.7, 311.1, 329.6, /* 66~76 */
    349.2, 370.0, 392.0, 415.3, 440.0, 466.2, 493.9, 523.3, 554.4, /* 77~85 */
    1174.66, 1244.51, 1318.51, 1396.91, 1479.98, 1567.98, 1661.22, 1760.0, 1864.66,
    1975.53, /* 86~95 */
    2093.0, 2217.46, 2349.32, 2489.02, 2637.02, 2793.83, 2959.96, 3135.96, 3322.44,
    3520.0, /* 96~105 */
    3729.31, 3951.07, 4186.01, 4434.92, 4698.64, 4978.03, 5274.04, 5587.65, 5919.91,
    6271.93, /* 106~115 */
    6644.88, 7040.0, 7458.62, 7902.13, 8372.02, 8869.84, 9397.27, 9956.06, 10548.08,
    11175.3, /* 116~125 */
    11839.82, 12543.85, /* 126~127 */
];

struct Note {
    channel: u8,
    note: u8,
    delay: u16,
}

const MIDI_CONTENT: [Note; 896] = [
    Note {
        channel: 0,
        note: 71,
        delay: 26,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 465,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 26,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 465,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 26,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 2334,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 125,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 465,
    },
    Note {
        channel: 0,
        note: 66,
        delay: 26,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 465,
    },
    Note {
        channel: 0,
        note: 64,
        delay: 26,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 465,
    },
    Note {
        channel: 0,
        note: 62,
        delay: 26,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 2334,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 125,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 465,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 26,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 465,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 26,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 465,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 26,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 698,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 38,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 73,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 465,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 26,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 465,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 26,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 73,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 698,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 38,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1399,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 75,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 465,
    },
    Note {
        channel: 0,
        note: 66,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 480,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 27,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 118,
    },
    Note {
        channel: 0,
        note: 64,
        delay: 29,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 506,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 29,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 62,
        delay: 30,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1581,
    },
    Note {
        channel: 0,
        note: 64,
        delay: 85,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 261,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 16,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 81,
    },
    Note {
        channel: 0,
        note: 62,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 3617,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 192,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 902,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 49,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 91,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 88,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1807,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 97,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1807,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 97,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1355,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 73,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 337,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 19,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 111,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 7,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1807,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 97,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 902,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 49,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 902,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 49,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 337,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 19,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 111,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 7,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1610,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 87,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 266,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 16,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 266,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 16,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 266,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 16,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 266,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 16,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 266,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 16,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 266,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 16,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 385,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 28,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 422,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 902,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 49,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 91,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 88,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1807,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 97,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1807,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 97,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1355,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 73,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 337,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 19,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 111,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 7,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1807,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 97,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 902,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 49,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 902,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 49,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 337,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 19,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 111,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 7,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1422,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 77,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 235,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 235,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 133,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 231,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 272,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 27,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 472,
    },
    Note {
        channel: 0,
        note: 91,
        delay: 1322,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 480,
    },
    Note {
        channel: 0,
        note: 93,
        delay: 27,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 239,
    },
    Note {
        channel: 0,
        note: 95,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 239,
    },
    Note {
        channel: 0,
        note: 93,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 239,
    },
    Note {
        channel: 0,
        note: 91,
        delay: 14,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 239,
    },
    Note {
        channel: 0,
        note: 90,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 497,
    },
    Note {
        channel: 0,
        note: 88,
        delay: 28,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 497,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 28,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 3997,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 836,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 45,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 836,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 45,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 836,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 2259,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 121,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 62,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 66,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 2259,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 121,
    },
    Note {
        channel: 0,
        note: 90,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 88,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 90,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 88,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 85,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 85,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 83,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 71,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 119,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1242,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 67,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 378,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 71,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 78,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 62,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 66,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1807,
    },
    Note {
        channel: 0,
        note: 62,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 319,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 107,
    },
    Note {
        channel: 0,
        note: 62,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 902,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 49,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 89,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 94,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 91,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 89,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 89,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1807,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 97,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 119,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 355,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 208,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 31,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 62,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1807,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 97,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 89,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1355,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 62,
        delay: 73,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 337,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 19,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 111,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 62,
        delay: 7,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 69,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 81,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1807,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 97,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 676,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 37,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 89,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 25,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 62,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 902,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 49,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 902,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 49,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 62,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 67,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 224,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 60,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 106,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 32,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 83,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 120,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 83,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 18,
    },
    Note {
        channel: 0,
        note: 65,
        delay: 111,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 550,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 23,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 13,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 95,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 89,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 472,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 70,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 74,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 515,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 72,
        delay: 575,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 515,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 76,
        delay: 29,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 88,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 515,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 79,
        delay: 29,
    },
    Note {
        channel: 0,
        note: 84,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 88,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 91,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 450,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 77,
        delay: 27,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 237,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 89,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 82,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1807,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 86,
        delay: 97,
    },
    Note {
        channel: 0,
        note: 89,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1712,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 95,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 89,
        delay: 97,
    },
    Note {
        channel: 0,
        note: 94,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 98,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 101,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 1807,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
    Note {
        channel: 0,
        note: 0,
        delay: 0,
    },
];
