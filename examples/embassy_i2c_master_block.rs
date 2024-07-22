#![no_std]
#![no_main]

use embedded_io::Write;
use hal::i2c::AnyI2c;
use heapless::String;
use py32f030_hal::{self as hal, mode::Blocking};

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());
    let gpioa = p.GPIOA.split();

    let sda = gpioa.PA9;
    let scl = gpioa.PA10;

    let i2c1 = AnyI2c::<_, Blocking>::new(p.I2C, sda, scl, Default::default());

    defmt::info!("usart start...");
    let buf: String<20> = "hello rust\r\n".into();
    loop {
        defmt::info!("send: {} ", buf.as_bytes());
        cortex_m::asm::delay(1000 * 1000 * 10);
    }
}
