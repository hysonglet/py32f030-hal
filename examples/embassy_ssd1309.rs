#![no_std]
#![no_main]

use embedded_hal_027::digital::v2::OutputPin;
use py32f030_hal::gpio::{Output, PinIoType, PinSpeed};
use py32f030_hal::mode::Blocking;

use py32f030_hal::{self as hal, clock};

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::i2c::{AnyI2c, Config};
use hal::timer::advanced_timer::AnyTimer;

use {defmt_rtt as _, panic_probe as _};

use display_interface_i2c::I2CInterface;
use embedded_graphics::{
    image::{Image, ImageRawLE},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
};
use ssd1309::{prelude::*, Builder};

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
    let i2c1 = AnyI2c::<_, Blocking>::new(p.I2C, scl, sda, config).unwrap();
    let master = i2c1.as_master();

    let timer = AnyTimer::<_, Blocking>::new(p.TIM1).unwrap();
    let mut counter = timer.as_counter();

    _spawner.spawn(run()).unwrap();

    let i2c_interface = I2CInterface::new(master, 0x3C, 0x40);
    let mut disp: GraphicsMode<_> = Builder::new().connect(i2c_interface).into();
    disp.reset(&mut lcd_rst, &mut counter).unwrap();

    disp.init().unwrap();
    disp.flush().unwrap();

    // Top side
    disp.set_pixel(0, 0, 1);
    disp.set_pixel(1, 0, 1);
    disp.set_pixel(2, 0, 1);
    disp.set_pixel(3, 0, 1);

    // Right side
    disp.set_pixel(3, 0, 1);
    disp.set_pixel(3, 1, 1);
    disp.set_pixel(3, 2, 1);
    disp.set_pixel(3, 3, 1);

    // Bottom side
    disp.set_pixel(0, 3, 1);
    disp.set_pixel(1, 3, 1);
    disp.set_pixel(2, 3, 1);
    disp.set_pixel(3, 3, 1);

    // Left side
    disp.set_pixel(0, 0, 1);
    disp.set_pixel(0, 1, 1);
    disp.set_pixel(0, 2, 1);
    disp.set_pixel(0, 3, 1);

    disp.flush().unwrap();

    Line::new(Point::new(8, 16 + 16), Point::new(8 + 16, 16 + 16))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut disp)
        .unwrap();

    Line::new(Point::new(8, 16 + 16), Point::new(8 + 8, 16))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut disp)
        .unwrap();

    Line::new(Point::new(8 + 16, 16 + 16), Point::new(8 + 8, 16))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut disp)
        .unwrap();

    Rectangle::with_corners(Point::new(48, 16), Point::new(48 + 16, 16 + 16))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut disp)
        .unwrap();

    Circle::new(Point::new(88, 16), 16)
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut disp)
        .unwrap();

    disp.flush().unwrap();

    defmt::info!("{}", clock::sys_core_clock());

    loop {
        Timer::after_secs(2).await;
    }
}
