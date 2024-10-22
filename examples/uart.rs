#![no_std]
#![no_main]

use embedded_io::Write;
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

    let usart = AnyUsart::new(p.USART1, Some(rx), Some(tx), None, None, Default::default());

    let (_, mut tx) = usart.split();

    defmt::info!("usart start...");
    let buf: String<20> = "hello rust\r\n".into();
    loop {
        // 使用标准接口来发送串口数据
        let _ = write!(tx, "example for usart\r\n");

        // 使用自定义的驱动接口发送串口数据
        let _ = tx.write(buf.as_bytes());

        defmt::info!("send: {} ", buf.as_bytes());
        cortex_m::asm::delay(1000 * 1000 * 10);
    }
}
