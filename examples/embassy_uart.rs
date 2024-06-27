#![no_std]
#![no_main]

use core::fmt::Write;
use defmt::Debug2Format;
use hal::{mode::Blocking, usart::FlexUsart};
use heapless::String;
use py32f030_hal as hal;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());
    let gpioa = p.GPIOA.split();
    let rx = gpioa.PA9;
    let tx = gpioa.PA10;

    let usart = FlexUsart::new(p.USART1, Some(rx), Some(tx), Default::default());

    let (rx, tx) = usart.split();

    defmt::info!("usart start...");
    let mut buf: String<20> = String::new();
    let mut cnt: u32 = 0;
    let mut r_buf: [u8; 10] = [0; 10];
    loop {
        cnt += 1;
        buf.clear();
        let _ = write!(&mut buf, "{}\r\n", cnt);
        tx.write_bytes_blocking(buf.as_bytes());
        rx.read_blocking(&mut r_buf);
        cortex_m::asm::delay(1000 * 1000 * 10);
        defmt::info!("send: {} ", buf.as_bytes());
        defmt::info!("recv: {}", r_buf);
    }
}
