#![no_std]
#![allow(non_camel_case_types)]
#![allow(clippy::uninit_assumed_init)]
#![allow(internal_features)]
#![feature(core_intrinsics)]
#![allow(non_snake_case)]
#![feature(async_closure)]

use config::SysClockSource;
use mcu::Peripherals;
pub use PY32f030xx_pac as pac;

pub mod adc;
pub mod bit;
pub mod clock;
pub mod crc;
pub mod delay;
pub mod dma;
// pub mod dwt;
#[cfg(feature = "embassy")]
mod embassy;
pub mod exti;
pub mod flash;
pub mod gpio;
pub mod i2c;
#[cfg(not(feature = "embassy"))]
pub mod interrupt;
pub mod iwdg;
mod macro_def;
pub mod mcu;
pub(crate) mod pwr;
pub mod rtc;
pub mod spi;
pub mod syscfg;
pub mod timer;
pub mod usart;

#[doc(hidden)]
pub mod prelude {
    pub use embedded_hal::digital::{InputPin as _, OutputPin as _, StatefulOutputPin as _};
}

pub mod config {

    /// 系统时钟选择
    #[derive(Default)]
    pub enum SysClockSource {
        /// 8M
        #[default]
        HSI,
        /// 24M
        HSE,
        /// 16M
        PLL_HSI,
    }

    /// 系统初始化配置
    ///  - 指定默认运行的时钟
    #[derive(Default)]
    pub struct Config {
        /// 默认时钟配置
        pub sys_clk: SysClockSource,
    }

    impl Config {
        /// 设置系统时钟
        pub fn sys_clk(self, sys_clk: SysClockSource) -> Self {
            Self { sys_clk }
        }
    }
}

/// 初始化时钟运行环境、系统、基本的外设
pub fn init(config: config::Config) -> Peripherals {
    let peripherals = Peripherals::take();
    cortex_m::asm::delay(1000 * 1000 * 5);
    match config.sys_clk {
        // 使用外部时钟源
        SysClockSource::HSE => {
            clock::SysClock::<clock::HSE>::config().unwrap();
        }
        // 使用内部时钟源
        SysClockSource::HSI => {
            clock::SysClock::<clock::HSIDiv<1>>::config().unwrap();
        }
        // 使用内部的PLL时钟源
        SysClockSource::PLL_HSI => {
            // HSI::set_hz(clock::HsiHz::MHz8);
            clock::SysClock::<clock::PLL<clock::HSI>>::config().unwrap();
        }
    }

    // 启用异步os
    #[cfg(feature = "embassy")]
    embassy::init();

    peripherals
}

/// 定义外设工作模式，阻塞或异步方式
pub mod mode {
    trait Sealed {}

    /// 外设的工作模式
    #[allow(private_bounds)]
    pub trait Mode: Sealed {
        fn is_async() -> bool;
    }

    /// 阻塞模式对象
    pub struct Blocking;
    /// 异步模式对象
    pub struct Async;

    impl Sealed for Blocking {}
    impl Mode for Blocking {
        fn is_async() -> bool {
            false
        }
    }

    impl Sealed for Async {}
    impl Mode for Async {
        fn is_async() -> bool {
            true
        }
    }
}
