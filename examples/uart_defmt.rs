#![no_std]
#![no_main]

use py32f030_hal::{self as hal, mcu::peripherals::USART1, mode::Blocking, usart::AnyUsart};
use static_cell::StaticCell;

static SERIAL: StaticCell<AnyUsart<'static, USART1, Blocking>> = StaticCell::new();

// use defmt_rtt as _;
use panic_probe as _;
use {defmt::info, defmt_serial as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());
    let gpioa = p.GPIOA.split();

    let tx = gpioa.PA9;
    let rx = gpioa.PA10;

    let usart = AnyUsart::new(p.USART1, Some(rx), Some(tx), None, None, Default::default());

    defmt_serial::defmt_serial(SERIAL.init(usart));

    let mut cnt = 0;
    loop {
        // https://github.com/gauteh/defmt-serial
        info!("hello world {} {}", 123, cnt);
        cnt += 1;
        cortex_m::asm::delay(1000 * 1000 * 10);
    }
}
