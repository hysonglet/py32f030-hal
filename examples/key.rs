#![no_std]
#![no_main]

use defmt_rtt as _;
use hal::InputPin;
// use embedded_hal::digital::v2::StatefulOutputPin;
use panic_probe as _;

use hal::clock::{self, Mco, HSI};
use hal::gpio::gpioa;
use hal::gpio::{Input, PinPullUpDown, PinSpeed};
use py32f030_hal as hal;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("examples: clock");

    cortex_m::asm::delay(1000 * 1000 * 5);
    let _sysclk = clock::Sysclock::<clock::PLL<HSI>>::config().unwrap();

    Mco::select(clock::McoSelect::SysClk, clock::McoDIV::DIV1);

    let gpioa = gpioa::GPIOA.split();

    let key = Input::new(gpioa.PA12, PinPullUpDown::PullUp, PinSpeed::Low);

    cortex_m::asm::delay(1000 * 1000 * 5);
    defmt::info!("freq: {}MHZ", clock::sys_core_clock() / 1000 / 1000);
    let mut cnt = 0;
    loop {
        cortex_m::asm::delay(1000 * 1000 * 5);
        cnt += 1;
        defmt::info!("cnt: {}", cnt);

        defmt::info!("{}", key.is_low());
    }
}
