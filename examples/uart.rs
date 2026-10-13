#![no_std]
#![no_main]

use heapless::String;
use py32f030_hal::{self as hal, prelude::*, usart::AnyUsart};
use {defmt::info, defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());
    let gpioa = p.GPIOA.split();

    let tx = gpioa.PA9;
    let rx = gpioa.PA10;

    let usart = AnyUsart::new(p.USART1, Some(rx), Some(tx), None, None, Default::default());

    let (_, mut tx) = usart.split();

    info!("usart start...");
    let buf: String<20> = "hello rust\r\n".into();
    loop {
        // Use the standard interface to send serial data.
        let _ = write!(tx, "example for usart\r\n");

        // Use the custom driver interface to send serial data.
        let _ = tx.write(buf.as_bytes());

        info!("send: {} ", buf.as_bytes());
        cortex_m::asm::delay(1000 * 1000 * 10);
    }
}
