#![no_std]
#![no_main]

use defmt::{info, Debug2Format};
use hal::syscfg;
use hal::usart::AnyUsart;
use py32f030_hal::{self as hal, mode::Blocking, prelude::*};

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());

    syscfg::syscfg::open();

    let gpioa = p.GPIOA.split();

    let rx = gpioa.PA10;
    let tx = gpioa.PA9;

    let usart: AnyUsart<_, Blocking> =
        AnyUsart::new(p.USART1, Some(rx), Some(tx), None, None, Default::default());

    let (rx, mut tx) = usart.split();

    info!("usart start...");

    let mut rx_buf: [u8; 10] = [0; 10];
    let _ = write!(tx, "example for usart\r\n");
    loop {
        let cnt = rx.read_blocking(&mut rx_buf);
        info!("recv: cnt: {} {}", Debug2Format(&cnt), rx_buf[0..cnt]);

        let _ = tx.write(rx_buf[0..cnt].as_ref());
    }
}
