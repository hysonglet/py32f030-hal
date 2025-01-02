#[cfg(feature = "embassy")]
mod future;
mod hal;
mod pins;
mod types;

use core::convert::Infallible;

pub use types::*;

// use self::hal::sealed::Instance;
use crate::gpio::Pin;
use crate::gpio::{Input, PinLevel, PinPullUpDown, PinSpeed};
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
    pub fn new(
        pin: impl Peripheral<P = impl Pin> + 'd,
        pull: PinPullUpDown,
        speed: PinSpeed,
    ) -> Self {
        into_ref!(pin);

        Self {
            pin: Input::new(pin, pull, speed),
            _mode: PhantomData,
        }
    }

    /// 返回引脚的电平
    #[inline]
    fn get_level(&self) -> PinLevel {
        self.pin.read()
    }
}

impl<'d> ExtiInput<'d, Blocking> {
    /// 等待引脚电平变低
    #[inline]
    pub fn wait_for_low(&self) {
        while self.get_level() == PinLevel::High {}
    }

    /// 等待引脚电平变高
    #[inline]
    pub fn wait_for_high(&self) {
        while self.get_level() == PinLevel::Low {}
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
        if self.get_level() == PinLevel::Low {
            return;
        }

        future::ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Falling).await
    }

    /// 等待引脚电平变高
    pub async fn wait_for_high(&self) {
        if self.get_level() == PinLevel::High {
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

impl<'a, M: Mode> embedded_hal_027::digital::v2::InputPin for ExtiInput<'a, M> {
    type Error = Infallible;
    fn is_low(&self) -> Result<bool, Self::Error> {
        self.pin.is_low()
    }

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.pin.is_high()
    }
}
