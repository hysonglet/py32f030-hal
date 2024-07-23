#![no_std]
#![no_main]

use embedded_io::Write;
use hal::usart::FlexUsart;
use heapless::String;
use py32f030_hal as hal;

use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());

    let gpioa = p.GPIOA.split();
    let rx = gpioa.PA9;
    let tx = gpioa.PA10;

    let usart = FlexUsart::new(p.USART1, Some(rx), Some(tx), Default::default());

    let (_, mut tx) = usart.split();

    defmt::info!("usart start...");
    let buf: String<20> = "hello rust\r\n".into();

    loop {
        // 使用标准接口来发送串口数据
        let _ = write!(tx, "example for usart\r\n");

        // 使用自定义的驱动接口发送串口数据
        tx.write_bytes_blocking(buf.as_bytes());

        defmt::info!("send: {} ", buf.as_bytes());
        Timer::after_secs(5).await;
    }
}
