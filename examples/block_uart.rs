#![no_std]
#![no_main]

use defmt::Debug2Format;
use embedded_io::{Read, Write};
use hal::dma::AnyDma;
use hal::syscfg;
use hal::usart::AnyUsart;
use heapless::String;
use py32f030_hal::{self as hal, mode::Blocking};

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());

    syscfg::syscfg::open();

    let gpioa = p.GPIOA.split();

    let rx = gpioa.PA10;
    let tx = gpioa.PA9;

    let mut dma: AnyDma<_, _> = AnyDma::new(p.DMA);
    let [channel1, channel2, _] = dma.split();

    let usart: AnyUsart<_, Blocking> = AnyUsart::new(
        p.USART1,
        Some(rx),
        Some(tx),
        Some(channel1),
        Some(channel2),
        Default::default(),
    );

    let (mut rx, mut tx) = usart.split();

    defmt::info!("usart start...");
    let buf: String<20> = "hello rust\r\n".into();

    let mut rx_buf: [u8; 10] = [0; 10];

    loop {
        // let cnt = rx.read_blocking(&mut rx_buf);
        let cnt = rx.read_dma_idle_blocking(&mut rx_buf).unwrap();
        defmt::info!("recv: cnt: {} {}", Debug2Format(&cnt), rx_buf[0..cnt]);
        // tx.write_bytes_blocking(&rx_buf);

        // let cnt = rx.read_idle_blocking(&mut rx_buf);
        // defmt::info!("recv idle: cnt: {} {:x}", cnt, rx_buf[0..cnt]);

        // // 使用标准接口来发送串口数据
        let _ = write!(tx, "example for usart\r\n");

        // // 使用自定义的驱动接口发送串口数据
        let _ = tx.write(&rx_buf[0..cnt]);
        // let _ = tx.write(buf.as_bytes());

        // defmt::info!("send: {} ", buf.as_bytes());

        // hal::delay::delay_ms(1000);
    }
}
