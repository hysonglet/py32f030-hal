mod hal;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

pub enum AdvancedTimer {
    TIM1,
}

pub enum Channel {
    CH1,
    CH2,
    CH3,
    CH4,
}

#[derive(Debug)]
pub enum Error {}

/// 记数模式
pub enum CountMode {
    /// 向上计数
    Up,
    /// 向下计数
    Down,
    /// 中间计数
    Center,
}

pub struct BaseConfig {
    count_mode: CountMode,
    prescaler: u32,
    period: u32,
    repetition_counter: u32,
    auto_reload: bool,
}

pub struct Counter;
pub struct Capture;
pub struct Pwm;
pub struct Hall;
pub struct Motor;
