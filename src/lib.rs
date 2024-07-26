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
pub(crate) mod common;
pub mod delay;
pub mod dma;
#[cfg(feature = "embassy")]
mod embassy;
pub mod exti;
pub mod gpio;
pub mod i2c;
pub mod mcu;
pub mod timer;
pub mod usart;

mod macro_def;

pub mod config {

    /// 系统时钟选择
    #[derive(Default)]
    pub enum SysClockSource {
        #[default]
        HSI,
        HSE,
    }

    /// 系统初始化配置
    #[derive(Default)]
    pub struct Config {
        /// 默认时钟配置
        pub sys_clk: SysClockSource,
    }
}

pub fn init(config: config::Config) -> Peripherals {
    let peripherals = Peripherals::take();
    cortex_m::asm::delay(1000 * 1000 * 5);
    match config.sys_clk {
        SysClockSource::HSE => {
            clock::SysClock::<clock::HSE>::config().unwrap();
        }
        SysClockSource::HSI => {
            clock::SysClock::<clock::PLL<clock::HSI>>::config().unwrap();
        }
    }

    #[cfg(feature = "defmt")]
    defmt::info!("freq: {}MHZ", clock::sys_core_clock() / 1000 / 1000);

    #[cfg(feature = "embassy")]
    embassy::init();

    critical_section::with(|cs| {
        exti::init(cs);

        #[cfg(feature = "dma")]
        dma::init(cs);
    });

    peripherals
}

/// 外设工作模式
pub mod mode {
    trait Sealed {}

    /// 外设的工作模式
    #[allow(private_bounds)]
    pub trait Mode: Sealed {}

    /// 阻塞模式对象
    pub struct Blocking;
    /// 异步模式对象
    pub struct Async;

    impl Sealed for Blocking {}
    impl Mode for Blocking {}

    impl Sealed for Async {}
    impl Mode for Async {}
}
