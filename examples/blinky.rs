#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use hal::{
    clock::{self, HSI},
    gpio::{Floating, GpioA, GpioPin, Input, Output, PullUp},
};

use embedded_hal::digital::v2::{InputPin, OutputPin, ToggleableOutputPin};
use py32f030_hal as hal;

const LOOP_CNT: u32 = 200000;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");
    let key: GpioPin<GpioA, 12, Input<Floating>> = GpioPin::new();
    let mut led: GpioPin<GpioA, 11, Output<PullUp>> = GpioPin::new();

    cortex_m::asm::delay(1000 * 1000 * 5);
    let _sysclk = clock::Sysclock::<clock::PLL<HSI>>::config().unwrap();

    cortex_m::asm::delay(1000 * 1000 * 5);
    defmt::info!("freq: {}MHZ", clock::sys_core_clock() / 1000 / 1000);
    let mut cnt = 0;
    loop {
        cortex_m::asm::delay(1000 * 1000 * 5);
        cnt += 1;
        defmt::info!("cnt: {}", cnt);
        let _ = led.toggle();
        defmt::info!("{}", key.is_low());
    }
}
