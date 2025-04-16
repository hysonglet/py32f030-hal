mod counter;
#[cfg(feature = "embassy")]
mod future;
mod hal;
mod pins;
mod pwm;
mod types;

use core::marker::PhantomData;

pub use counter::Counter;
pub use pwm::Pwm;
pub use types::*;

use crate::{
    clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex, PeripheralInterrupt},
    mode::Mode,
};
use embassy_hal_internal::{into_ref, Peripheral};

pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

/// 高级定时器
#[derive(PartialEq)]
pub enum Timer {
    TIM1,
    TIM3,
    TIM14,
    TIM16,
    TIM17,
}

impl PeripheralIdToClockIndex for Timer {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::TIM1 => PeripheralClockIndex::TIM1,
            Self::TIM3 => PeripheralClockIndex::TIM3,
            Self::TIM14 => PeripheralClockIndex::TIM14,
            Self::TIM16 => PeripheralClockIndex::TIM16,
            Self::TIM17 => PeripheralClockIndex::TIM17,
        }
    }
}

impl PeripheralInterrupt for Timer {
    fn interrupt(&self) -> crate::pac::interrupt {
        match *self {
            Self::TIM1 => crate::pac::interrupt::TIM1_BRK_UP_TRG_COM,
            Self::TIM3 => crate::pac::interrupt::TIM3,
            Self::TIM14 => crate::pac::interrupt::TIM14,
            Self::TIM16 => crate::pac::interrupt::TIM16,
            Self::TIM17 => crate::pac::interrupt::TIM17,
        }
    }
}

pub struct Capture;
pub struct Hall;
pub struct Motor;

macro_rules! impl_sealed_timer {
    (
        $peripheral: ident, $timer_id: ident
    ) => {
        impl hal::sealed::Instance for crate::mcu::peripherals::$peripheral {
            fn id() -> Timer {
                Timer::$timer_id
            }
        }
        impl Instance for crate::mcu::peripherals::$peripheral {}
    };
}

pub struct AnyTimer<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

impl_sealed_timer!(TIM1, TIM1);
impl_sealed_timer!(TIM3, TIM3);
impl_sealed_timer!(TIM14, TIM14);
impl_sealed_timer!(TIM16, TIM16);
impl_sealed_timer!(TIM17, TIM17);

impl<'d, T: Instance, M: Mode> AnyTimer<'d, T, M> {
    /// 新建一个 timer
    pub fn new(_timer: impl Peripheral<P = T>) -> Result<Self, Error> {
        into_ref!(_timer);

        // 开启外设时钟
        T::id().clock().open();

        Ok(Self {
            _t: PhantomData,
            _m: PhantomData,
        })
    }

    /// 返回定时器外设的时钟
    pub fn get_timer_clock() -> u32 {
        T::get_time_pclk()
    }

    /// 转换成计数模式
    pub fn as_counter(self) -> Counter<'d, T, M> {
        Counter::new()
    }

    /// 转换成pwm模式
    pub fn as_pwm(self) -> Result<Pwm<'d, T>, Error> {
        Pwm::new()
    }
}

// 定义一个 定时器引脚 的trait
pin_af_for_instance_def!(TimerChannel1Pin, Instance);
pin_af_for_instance_def!(TimerChannel1NPin, Instance);
pin_af_for_instance_def!(TimerChannel2Pin, Instance);
pin_af_for_instance_def!(TimerChannel2NPin, Instance);
pin_af_for_instance_def!(TimerChannel3Pin, Instance);
pin_af_for_instance_def!(TimerChannel3NPin, Instance);
pin_af_for_instance_def!(TimerChannel4Pin, Instance);
pin_af_for_instance_def!(TimerBkInPin, Instance);
pin_af_for_instance_def!(TimerEtrPin, Instance);
