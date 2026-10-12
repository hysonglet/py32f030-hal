//! 统一 APB 定时器模块
//!
//! TIM1 / TIM3 / TIM14 / TIM16 共用同一套 `Counter` / `Pwm` / `OutputCompare` /
//! `InputCapture` / `Encoder` / `OnePulse` 接口。定时器**专有**的能力（互补输出、
//! 刹车与死区、重复计数、编码器、单脉冲、中心对齐、主模式 TRGO、TI1 重映射）通过
//! [`capability`] 中的标记 trait 在**编译期**门控：对不具备该硬件的定时器调用
//! 对应方法会直接编译失败，而不是运行时 panic。
//!
//! LPTIM 因寄存器集/事件模型/时钟选择差异过大，保留为独立的 [`lptim`] 子模块。

mod capability;
mod counter;
mod encoder;
#[cfg(feature = "embassy")]
mod future;
mod ic;
mod instance;
mod oc;
mod one_pulse;
mod pins;
mod pwm;
mod types;

pub mod lptim;

pub use capability::*;
pub use counter::Counter;
pub use encoder::Encoder;
pub use ic::{InputCapture, InputCaptureConfig};
pub use oc::OutputCompare;
pub use one_pulse::{OnePulse, OnePulseConfig};
pub use pwm::Pwm;
pub use types::*;

use core::marker::PhantomData;
use crate::clock::peripheral::PeripheralIdToClockIndex;
use crate::mode::Mode;
use embassy_hal_internal::{into_ref, Peripheral};

/// 统一定时器实例约束（密封：用户无法实现）
///
/// 要求 `ChannelCount` 以便高层 API 用 `T::CHANNELS` 校验通道是否可用。
#[allow(private_bounds)]
pub trait Instance:
    Peripheral<P = Self> + instance::sealed::Instance + ChannelCount + 'static + Send
{
}

impl<T> Instance for T where
    T: Peripheral<P = T> + instance::sealed::Instance + ChannelCount + 'static + Send
{
}

/// 定时器句柄：负责开启/关闭外设时钟，并转换为具体功能
pub struct AnyTimer<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

impl<'d, T: Instance, M: Mode> AnyTimer<'d, T, M> {
    /// 新建一个定时器（开启外设时钟）
    pub fn new(_timer: impl Peripheral<P = T>) -> Result<Self, Error> {
        into_ref!(_timer);
        T::id().clock().open();
        Ok(Self {
            _t: PhantomData,
            _m: PhantomData,
        })
    }

    /// 返回定时器外设时钟
    pub fn get_timer_clock() -> u32 {
        T::get_time_pclk()
    }

    /// 转换成计数器 / 延时
    pub fn as_counter(self) -> Counter<'d, T, M> {
        Counter::new()
    }

    /// 转换成 PWM
    pub fn as_pwm(self) -> Pwm<'d, T> {
        Pwm::new()
    }

    /// 转换成输入捕获
    pub fn as_input_capture(self) -> InputCapture<'d, T> {
        InputCapture::new()
    }

    /// 转换成输出比较
    pub fn as_output_compare(self) -> OutputCompare<'d, T> {
        OutputCompare::new()
    }
}

/// 单脉冲：仅 TIM1 / TIM3 / TIM16
impl<'d, T: Instance + WithOnePulse, M: Mode> AnyTimer<'d, T, M> {
    pub fn as_one_pulse(self, cfg: OnePulseConfig, mode: OnePulseMode) -> OnePulse<'d, T> {
        OnePulse::new(cfg, mode)
    }
}

/// 编码器：仅 TIM1 / TIM3
impl<'d, T: Instance + WithEncoder, M: Mode> AnyTimer<'d, T, M> {
    pub fn as_encoder(self) -> Encoder<'d, T> {
        Encoder::new()
    }
}

impl<'d, T: Instance, M: Mode> Drop for AnyTimer<'d, T, M> {
    fn drop(&mut self) {
        T::stop();
        T::id().clock().close();
    }
}

//////////////////////// 引脚 AF 绑定 ////////////////////////

pin_af_for_instance_def!(TimerChannel1Pin, Instance);
pin_af_for_instance_def!(TimerChannel1NPin, Instance);
pin_af_for_instance_def!(TimerChannel2Pin, Instance);
pin_af_for_instance_def!(TimerChannel2NPin, Instance);
pin_af_for_instance_def!(TimerChannel3Pin, Instance);
pin_af_for_instance_def!(TimerChannel3NPin, Instance);
pin_af_for_instance_def!(TimerChannel4Pin, Instance);
pin_af_for_instance_def!(TimerBkInPin, Instance);
pin_af_for_instance_def!(TimerEtrPin, Instance);

//////////////////////// 兼容旧路径别名 ////////////////////////
// 合并前每个定时器是独立模块（timer::advanced_timer / timer::general_purpose_timer /
// timer::timer14 / timer::timer16）。这里保留同名别名，使既有代码无需改动即可继续编译。

/// 兼容 `timer::advanced_timer::*`（原 TIM1 模块）
pub mod advanced_timer {
    pub use super::*;
}

/// 兼容 `timer::general_purpose_timer::*`（原 TIM3 模块）
pub mod general_purpose_timer {
    pub use super::*;
}

/// 兼容 `timer::timer14::*`
pub mod timer14 {
    pub use super::*;
}

/// 兼容 `timer::timer16::*`
pub mod timer16 {
    pub use super::*;
}
