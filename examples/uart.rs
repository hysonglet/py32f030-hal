#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use hal::clock::{self, Mco, SysClkSelect, HSE, HSI};
use hal::gpio::{Af, GpioA, GpioPin, PinAF, PinSpeed, PullUp, PA10, PA9};
use py32f030_hal as hal;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("examples: clock");

    cortex_m::asm::delay(1000 * 1000 * 5);
    // let sysclk = clock::Sysclock::<clock::HSIDiv<1>>::config().unwrap();
    // let sysclk = clock::Sysclock::<clock::HSE>::config().unwrap();
    // let sysclk = clock::Sysclock::<clock::PLL<HSE>>::config().unwrap();
    let sysclk = clock::Sysclock::<clock::PLL<HSI>>::config().unwrap();

    Mco::select(clock::McoSelect::SysClk, clock::McoDIV::DIV1);

    let tx_pin: GpioPin<GpioA, 9, Af<PullUp>> = GpioPin::new();
    tx_pin.speed(PinSpeed::VeryHigh);
    let rx_pin: GpioPin<GpioA, 10, Af<PullUp>> = GpioPin::new();
    rx_pin.speed(PinSpeed::VeryHigh);

    cortex_m::asm::delay(1000 * 1000 * 5);
    defmt::info!("freq: {}MHZ", clock::sys_core_clock() / 1000 / 1000);
    let mut cnt = 0;
    loop {
        cortex_m::asm::delay(1000 * 1000 * 5);
        cnt += 1;
        defmt::info!("cnt: {}", cnt);
    }
}
