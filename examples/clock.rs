#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use hal::clock::{self, Mco};
use hal::gpio::{Af, PinAF};
use py32f030_hal::gpio::{Output, PinIoType, Speed};
use py32f030_hal::{self as hal, prelude::*};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("examples: clock");
    let p = hal::init(Default::default());
    let gpioa = p.GPIOA.split();

    let _mco_pin = Af::new(gpioa.PA1, PinAF::AF15, Speed::VeryHigh, PinIoType::PullUp);
    Mco::select(clock::McoSelect::SysClk, clock::McoDIV::DIV1);

    let mut led = Output::new(gpioa.PA10, PinIoType::PullUp, Speed::VeryHigh);

    cortex_m::asm::delay(1000 * 1000 * 5);
    // let _sysclk = clock::SysClock::<clock::HSIDiv<1>>::config().unwrap();
    // let _sysclk = clock::SysClock::<clock::HSE>::config().unwrap();

    // let _sysclk = clock::SysClock::<clock::PLL<clock::HSE>>::config().unwrap();

    // PA1 输出 16M
    let _sysclk = clock::SysClock::<clock::PLL<clock::HSI>>::config().unwrap();

    cortex_m::asm::delay(1000 * 5);
    defmt::info!("freq: {}MHZ", clock::sys_core_clock() / 1000 / 1000);

    loop {
        cortex_m::asm::delay(1000 * 1000 * 10);
        defmt::info!("8888");

        let _ = led.toggle();
    }
}
