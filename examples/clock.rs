#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use hal::clock::{self, Mco, SysClkSelect, HSE, HSI};
use hal::gpio::PinAF;
use hal::{hal::digital::v2::ToggleableOutputPin, InputPin, OutputPin};
use py32f030_hal as hal;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("examples: clock");

    let port = hal::gpio::GPIOA::Port;
    let p = port.split();

    let mco = p.PA1.into_periph();
    let _ = mco.set_af(PinAF::AF15);

    cortex_m::asm::delay(1000 * 1000 * 5);
    // let sysclk = clock::Sysclock::<clock::HSIDiv<1>>::config().unwrap();
    // let sysclk = clock::Sysclock::<clock::HSE>::config().unwrap();
    // let sysclk = clock::Sysclock::<clock::PLL<HSE>>::config().unwrap();
    let sysclk = clock::Sysclock::<clock::PLL<HSI>>::config().unwrap();

    Mco::select(clock::McoSelect::SysClk, clock::McoDIV::DIV1);

    cortex_m::asm::delay(1000 * 1000 * 5);
    defmt::info!("freq: {}MHZ", clock::sys_core_clock() / 1000 / 1000);
    let mut cnt = 0;
    loop {
        cortex_m::asm::delay(1000 * 1000 * 5);
        cnt += 1;
        defmt::info!("cnt: {}", cnt);
    }
}
