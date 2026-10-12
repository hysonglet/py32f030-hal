#[cfg(feature = "embassy")]
mod future;
mod hal;
mod pins;
mod types;

use core::convert::Infallible;
use embedded_hal::digital::PinState;

pub use types::*;

// 内部寄存器访问标记类型（位于 hal.rs::sealed，始终编译）
use crate::exti::hal::sealed::{ExtiImpl as ExtiReg, Instance as _};

// use self::hal::sealed::Instance;
use crate::gpio::Pin;
use crate::gpio::{Input, Pull, Speed};
#[cfg(feature = "embassy")]
use crate::mode::Async;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};

// pub trait ExitPin:
//     Peripheral<P = Self> + sealed::Instance + crate::gpio::Pin + 'static + Send
// {
//     fn set_pin_select(&self) {
//         Self::exit_channle_select(self.pin().into(), self.port().into());
//     }
// }

pub struct ExtiInput<'d, M: Mode> {
    pin: Input<'d>,
    _mode: PhantomData<M>,
}

impl<'d, M: Mode> Unpin for ExtiInput<'d, M> {}

impl<'d, M: Mode> ExtiInput<'d, M> {
    pub fn new(pin: impl Peripheral<P = impl Pin> + 'd, pull: Pull, speed: Speed) -> Self {
        into_ref!(pin);

        Self {
            pin: Input::new(pin, pull, speed),
            _mode: PhantomData,
        }
    }

    /// 返回引脚的电平
    #[inline]
    fn get_level(&self) -> PinState {
        self.pin.read()
    }
}

impl<'d> ExtiInput<'d, Blocking> {
    /// 等待引脚电平变低
    #[inline]
    pub fn wait_for_low(&self) {
        while self.get_level() == PinState::High {}
    }

    /// 等待引脚电平变高
    #[inline]
    pub fn wait_for_high(&self) {
        while self.get_level() == PinState::Low {}
    }

    /// 等待上升沿信号
    #[inline]
    pub fn wait_for_rising_edge(&self) {
        self.wait_for_low();
        self.wait_for_high();
    }

    /// 等待下降沿信号
    #[inline]
    pub fn wait_for_falling_edge(&self) {
        self.wait_for_high();
        self.wait_for_low();
    }

    /// 等待上升沿和下降沿信号
    #[inline]
    pub fn wait_for_any_edge(&self) {
        let last = self.get_level();
        while last == self.get_level() {}
    }
}

#[cfg(feature = "embassy")]
impl<'d> ExtiInput<'d, Async> {
    /// 等待引脚电平变低
    pub async fn wait_for_low(&self) {
        if self.get_level() == PinState::Low {
            return;
        }

        future::ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Falling).await
    }

    /// 等待引脚电平变高
    pub async fn wait_for_high(&self) {
        if self.get_level() == PinState::High {
            return;
        }
        future::ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Rising).await
    }

    /// 等待上升沿信号
    pub async fn wait_for_rising_edge(&self) {
        future::ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Rising).await
    }

    /// 等待下降沿信号
    pub async fn wait_for_falling_edge(&self) {
        future::ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Falling).await
    }

    /// 等待上升沿和下降沿信号
    pub async fn wait_for_any_edge(&self) {
        future::ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::RisingFalling)
            .await
    }
}

impl<'a, M: Mode> embedded_hal::digital::ErrorType for ExtiInput<'a, M> {
    type Error = core::convert::Infallible;
}

impl<'a, M: Mode> embedded_hal::digital::InputPin for ExtiInput<'a, M> {
    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.pin.is_low()
    }

    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        self.pin.is_high()
    }
}

impl<'a, M: Mode> embedded_hal_027::digital::v2::InputPin for ExtiInput<'a, M> {
    type Error = Infallible;
    fn is_low(&self) -> Result<bool, Self::Error> {
        self.pin.is_low()
    }

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.pin.is_high()
    }
}

/// 直接操作 EXTI 外设（线使能、边沿、软件触发、挂起标志）
///
/// `ExtiInput` 是面向 GPIO 引脚的便捷封装；需要更底层控制（如软件触发、
/// 清除挂起、配置内部线 PVD/RTC/COMP）时，可使用本结构。
pub struct Exti;

impl Exti {
    /// 选择 GPIO 线与端口的映射（写 SYSCFG EXTICR），仅对 Line0–15 有效
    #[inline]
    pub fn select_gpio_source(line: Line, port: crate::gpio::GpioPort) {
        ExtiReg::exit_channle_select(line, port.into());
    }

    /// 设置边沿触发条件（上升/下降/双沿）
    #[inline]
    pub fn set_edge(line: Line, edge: Edge) {
        ExtiReg::line_ring_edge(line, edge.is_rising());
        ExtiReg::line_falling_edge(line, edge.is_falling());
    }

    /// 使能 / 屏蔽某线的中断与事件（IMR）
    #[inline]
    pub fn enable(line: Line, en: bool) {
        ExtiReg::line_pend_enable(line, en);
    }

    /// 软件触发某 EXTI 线（置 SWIER，硬件在 PR 中置位后自动清零）
    #[inline]
    pub fn software_trigger(line: Line) {
        ExtiReg::software_trigger(line);
    }

    /// 读取某线的挂起状态
    #[inline]
    pub fn is_pending(line: Line) -> bool {
        ExtiReg::pending_raw(line)
    }

    /// 清除某线的挂起标志（PR）
    #[inline]
    pub fn clear_pending(line: Line) {
        ExtiReg::clear_pending(line);
    }

    /// 等待某线挂起（阻塞轮询）
    #[inline]
    pub fn wait_pending(line: Line) {
        while !Self::is_pending(line) {}
    }
}
