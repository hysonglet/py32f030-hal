#![no_std]
#![no_main]

use defmt::Debug2Format;
// use embedded_io::Write;
use embedded_hal::digital::v2::OutputPin;
use hal::delay;
use hal::i2c::{AnyI2c, Config};
use py32f030_hal::delay::delay_ms;
use py32f030_hal::gpio::{Output, PinIoType, PinSpeed};
use py32f030_hal::{self as hal, mode::Blocking};

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());
    let gpioa = p.GPIOA.split();

    let mut lcd_rst = Output::new(gpioa.PA4, PinIoType::Pullup, PinSpeed::Low);
    let _ = lcd_rst.set_low();
    delay_ms(200);
    let _ = lcd_rst.set_high();

    let sda = gpioa.PA2;
    let scl = gpioa.PA3;

    let config = Config::default().speed(200_000);
    // 配置 200K的速度
    let i2c1 = AnyI2c::<_, Blocking>::new(p.I2C, scl, sda, config).unwrap();
    let master = i2c1.as_master();

    defmt::info!("i2c start...");
    let buf: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
    let mut r_buf: [u8; 2] = [0; 2];
    let mut cnt = 0;
    const SLAVE_DEVICE_ADDRESS: u8 = 0x78 >> 1;
    loop {
        // write from i2c
        let rst = master.write_block(SLAVE_DEVICE_ADDRESS, &buf);

        defmt::info!("write rst: {:?} ", Debug2Format(&rst));
        if rst.is_err() {
            master.clear_errors()
        }

        // read from i2c
        let rst = master.read_block(SLAVE_DEVICE_ADDRESS, &mut r_buf);
        defmt::info!("read rst: {:?} ", Debug2Format(&rst));

        delay::delay_ms(1000 * 1);
        defmt::info!("{}", cnt);
        cnt += 1;
    }
}
