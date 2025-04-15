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

#[derive(Default)]
pub struct ChannelOutputConfig {
    pub state: bool,
    pub polarity: bool,
    pub idle_state: bool,
}

pub struct ChannelConfig {
    pub mode: ChannelMode,
    pub clear: bool,
    pub fast: bool,
    pub preload: bool,
    /// Specifies the TIM Output Compare state.
    pub compare: u16,

    pub ch: Option<ChannelOutputConfig>,
    pub n_ch: Option<ChannelOutputConfig>,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            mode: ChannelMode::PWM1,
            clear: false,
            fast: false,
            preload: false,
            compare: 0,
            ch: None,
            n_ch: None,
        }
    }
}

impl ChannelConfig {
    pub fn mode(self, mode: ChannelMode) -> Self {
        Self { mode, ..self }
    }

    pub fn compare(self, compare: u16) -> Self {
        Self { compare, ..self }
    }

    pub fn ch(self, ch: ChannelOutputConfig) -> Self {
        Self {
            ch: Some(ch),
            ..self
        }
    }

    pub fn n_ch(self, n_ch: ChannelOutputConfig) -> Self {
        Self {
            n_ch: Some(n_ch),
            ..self
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
    pub fn as_pwm(self) -> Pwm<'d, T> {
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

use embedded_time::{rate::Fraction, Clock};

impl<'d, T: Instance, M: Mode> Clock for AnyTimer<'d, T, M> {
    type T = u32;
    const SCALING_FACTOR: Fraction = Fraction::new(1, 1);
    fn new_timer<Dur: embedded_time::duration::Duration>(
        &self,
        duration: Dur,
    ) -> embedded_time::Timer<
        embedded_time::timer::param::OneShot,
        embedded_time::timer::param::Armed,
        Self,
        Dur,
    >
    where
        Dur: embedded_time::fixed_point::FixedPoint,
    {
        todo!()
    }

    fn try_now(&self) -> Result<embedded_time::Instant<Self>, embedded_time::clock::Error> {
        todo!()
    }
}
