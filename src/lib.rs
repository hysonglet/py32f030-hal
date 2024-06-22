#![no_std]
#![allow(non_camel_case_types)]
#![allow(clippy::uninit_assumed_init)]

pub use embedded_hal as hal;
pub use embedded_hal::digital::v2::{InputPin, OutputPin};
pub use PY32f030xx_pac as pac;

pub mod clock;
pub mod common;
pub mod gpio;
pub mod macro_def;
// pub mod rcc;
