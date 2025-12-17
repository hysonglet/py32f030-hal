#![no_std]
#![no_main]

use defmt::info;
use embedded_time::rate::Baud;
use hal::dma::AnyDma;
use hal::syscfg;
use hal::usart::AnyUsart;
use hal::usart::Config;
use py32f030_hal::{self as hal, mode::Blocking, prelude::*};

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());

    syscfg::syscfg::open();

    let gpioa = p.GPIOA.split();

    let rx = gpioa.PA10;
    let tx = gpioa.PA9;

    let dma: AnyDma<_, _> = AnyDma::new(p.DMA);
    let [channel1, channel2, _] = dma.split();

    let usart: AnyUsart<_, Blocking> = AnyUsart::new(
        p.USART1,
        Some(rx),
        Some(tx),
        Some(channel1),
        Some(channel2),
        Config::default().baud_rate(Baud(9600)),
    );

    let (mut rx, mut tx) = usart.split();

    info!("usart start...");

    let mut rx_buf: [u8; 64] = [0; 64];

    let _ = write!(tx, "example for usart\r\n");

    let mut count = 0;
    loop {
        let cnt = rx.read_dma_idle_blocking(&mut rx_buf).unwrap();
        // 打印接收到的数据
        info!(
            "recv: cnt: {} {}",
            defmt::Debug2Format(&cnt),
            rx_buf[0..cnt]
        );
        count += cnt;
        info!("COUNT: {}", count);

        // // 将读取到的数据返回到串口
        // let _ = tx.write(&rx_buf[0..cnt]);
    }
}
