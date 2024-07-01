#![no_std]
#![allow(non_camel_case_types)]
#![allow(clippy::uninit_assumed_init)]
#![allow(internal_features)]
#![feature(core_intrinsics)]
#![allow(non_snake_case)]

use config::SysClockSource;
use mcu::Peripherals;
pub use PY32f030xx_pac as pac;

pub mod clock;
pub mod common;
#[cfg(feature = "embassy")]
pub mod embassy;
pub mod exit;
pub mod gpio;
pub mod mcu;
pub mod usart;

mod macro_def;

pub mod config {

    #[derive(Default)]
    pub enum SysClockSource {
        #[default]
        HSI,
        HSE,
    }

    #[derive(Default)]
    pub struct Config {
        pub sys_clk: SysClockSource,
    }
}

pub fn init(config: config::Config) -> Peripherals {
    let peripherals = Peripherals::take();
    cortex_m::asm::delay(1000 * 1000 * 5);
    match config.sys_clk {
        SysClockSource::HSE => {
            clock::Sysclock::<clock::HSE>::config().unwrap();
        }
        SysClockSource::HSI => {
            clock::Sysclock::<clock::PLL<clock::HSI>>::config().unwrap();
        }
    }

    #[cfg(feature = "defmt")]
    defmt::info!("freq: {}MHZ", clock::sys_core_clock() / 1000 / 1000);

    #[cfg(feature = "embassy")]
    embassy::init();

    peripherals
}

pub mod mode {
    trait Sealed {}

    #[allow(private_bounds)]
    pub trait Mode: Sealed {}

    pub struct Blocking;
    pub struct Async;

    impl Sealed for Blocking {}
    impl Mode for Blocking {}

    impl Sealed for Async {}
    impl Mode for Async {}
}
