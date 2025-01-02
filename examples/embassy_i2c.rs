#![no_std]
#![no_main]

use defmt::Debug2Format;
use embedded_hal::digital::OutputPin;
use py32f030_hal::gpio::{Output, PinIoType, PinSpeed};
use py32f030_hal::mode::Async;
use py32f030_hal::{self as hal};

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::i2c::{AnyI2c, Config};

// use panic_halt as _;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn run() {
    loop {
        Timer::after_secs(2).await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("i2c start...");
    let p = hal::init(Default::default());

    let gpioa = p.GPIOA.split();

    let mut lcd_rst = Output::new(gpioa.PA4, PinIoType::PullUp, PinSpeed::Low);
    let _ = lcd_rst.set_low();

    Timer::after_millis(1000).await;
    let _ = lcd_rst.set_high();

    let sda = gpioa.PA2;
    let scl = gpioa.PA3;
    let config = Config::default().speed(200_000);
    // 配置 200K的速度
    let i2c1 = AnyI2c::<_, Async>::new(p.I2C, scl, sda, config).unwrap();
    let mut master = i2c1.as_master();

    _spawner.spawn(run()).unwrap();
    let buf: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
    const SLAVE_DEVICE_ADDRESS: u8 = 0x3c;
    let mut cnt = 0;
    loop {
        let rst = master.write(SLAVE_DEVICE_ADDRESS, &buf).await;

        defmt::info!("rst: {:?} {}", Debug2Format(&rst), cnt);
        cnt += 1;
        Timer::after_secs(2).await;
    }
}
