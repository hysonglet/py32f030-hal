#![no_std]
#![no_main]

use cortex_m::register::psp::write;
use embedded_io::{Read, Write};
use hal::usart::AnyUsart;
use heapless::String;
use py32f030_hal as hal;

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());

    let gpioa = p.GPIOA.split();
    let rx = gpioa.PA9;
    let tx = gpioa.PA10;

    let usart = AnyUsart::new(p.USART1, Some(rx), Some(tx), Default::default());

    let (mut rx, mut tx) = usart.split();

    defmt::info!("usart start...");
    let buf: String<20> = "hello rust\r\n".into();

    let mut rx_buf: [u8; 10] = [0; 10];

    loop {
        let cnt = rx.read_blocking(&mut rx_buf);
        // defmt::info!("recv: cnt: {} {:x}", cnt, rx_buf);
        // tx.write_bytes_blocking(&rx_buf);

        // let cnt = rx.read_idle_blocking(&mut rx_buf);
        defmt::info!("recv idle: cnt: {} {:x}", cnt, rx_buf[0..cnt]);

        // // 使用标准接口来发送串口数据
        // let _ = write!(tx, "example for usart\r\n");

        // // 使用自定义的驱动接口发送串口数据
        // tx.write_bytes_blocking(buf.as_bytes());

        // defmt::info!("send: {} ", buf.as_bytes());

        // hal::delay::delay_ms(1000);
    }
}