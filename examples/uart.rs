#![no_std]
#![no_main]

use defmt::Debug2Format;
use defmt_rtt as _;
use hal::gpio::gpioa;
use panic_probe as _;

use hal::clock::{self, Mco, SysClkSelect, HSI};
use hal::gpio::{
    gpioa::{GpioA, PA10, PA9},
    Af, GpioPin, PinAF, PinSpeed, PullUp,
};
use hal::usart::{Uart, Usart1};
use py32f030_hal as hal;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("examples: clock");

    cortex_m::asm::delay(1000 * 1000 * 5);
    let sysclk = clock::Sysclock::<clock::PLL<HSI>>::config().unwrap();

    Mco::select(clock::McoSelect::SysClk, clock::McoDIV::DIV1);

    let tx_pin: GpioPin<GpioA, 9, Af<PullUp>> = GpioPin::new();
    tx_pin.af(gpioa::PA9::USART1_TX);
    tx_pin.speed(PinSpeed::VeryHigh);
    let rx_pin: GpioPin<GpioA, 10, Af<PullUp>> = GpioPin::new();
    rx_pin.speed(PinSpeed::VeryHigh);
    tx_pin.af(gpioa::PA10::USART1_RX);

    let uart = Usart1::new::<Uart>(Default::default());
    cortex_m::asm::delay(1000 * 1000 * 5);
    defmt::info!("freq: {}MHZ", clock::sys_core_clock() / 1000 / 1000);
    defmt::info!("pclk: {}MHZ", clock::sys_pclk() / 1000 / 1000);
    let mut cnt = 0;
    let buf = "123456\r\n";
    let mut read_buf: [u8; 64] = [0; 64];
    loop {
        cortex_m::asm::delay(1000 * 1000 * 5);
        cnt += 1;
        defmt::info!("cnt: {}", cnt);
        let rst = uart.write(buf.as_bytes(), 10000);
        defmt::info!("{:?}", Debug2Format(&rst));
        let rst = uart.read(&mut read_buf, 100000);
        defmt::info!("{:?}, {}", Debug2Format(&rst), read_buf);
    }
}
