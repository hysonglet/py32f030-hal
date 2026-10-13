#![no_std]
#![allow(non_camel_case_types)]
#![allow(clippy::uninit_assumed_init)]
#![allow(non_snake_case)]

use config::SysClockSource;
use mcu::Peripherals;
pub use PY32f030xx_pac as pac;

pub mod adc;
pub mod bit;
pub mod clock;
pub mod comp;
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
pub mod led;
#[cfg(not(feature = "embassy"))]
pub mod interrupt;
#[cfg(not(feature = "embassy"))]
extern crate alloc;
pub mod iwdg;
mod macro_def;
pub mod mcu;
pub mod pwr;
pub mod rtc;
pub mod spi;
pub mod syscfg;
pub mod timer;
pub mod usart;

#[doc(hidden)]
pub mod prelude {
    pub use crate::gpio::Pin as _;
    pub use embedded_hal::digital::{InputPin as _, OutputPin as _, StatefulOutputPin as _};
    pub use embedded_io::{Read as _, Write as _};
}

pub mod config {

    /// System clock source selection.
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

    /// System initialization configuration.
    ///  - - Specifies the clock to run by default.
    #[derive(Default)]
    pub struct Config {
        /// Default clock configuration.
        pub sys_clk: SysClockSource,
    }

    impl Config {
        /// Set the system clock.
        pub fn sys_clk(self, sys_clk: SysClockSource) -> Self {
            Self { sys_clk }
        }
    }
}

/// Initialize the clock runtime environment, system and basic peripherals.
pub fn init(config: config::Config) -> Peripherals {
    let peripherals = Peripherals::take();
    cortex_m::asm::delay(1000 * 1000 * 5);
    match config.sys_clk {
        // Use the external clock source.
        SysClockSource::HSE => {
            clock::SysClock::<clock::HSE>::config().unwrap();
        }
        // Use the internal clock source.
        SysClockSource::HSI => {
            clock::SysClock::<clock::HSIDiv<1>>::config().unwrap();
        }
        // Use the internal PLL clock source.
        SysClockSource::PLL_HSI => {
            // HSI::set_hz(clock::HsiHz::MHz8);
            clock::SysClock::<clock::PLL<clock::HSI>>::config().unwrap();
        }
    }

    // Enable the async OS (embassy).
    #[cfg(feature = "embassy")]
    embassy::init();

    peripherals
}

/// Defines the peripheral working mode: blocking or async.
pub mod mode {
    trait Sealed {}

    /// The peripheral working mode.
    #[allow(private_bounds)]
    pub trait Mode: Sealed {
        fn is_async() -> bool;
    }

    /// Blocking-mode marker.
    pub struct Blocking;
    /// Async-mode marker.
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
