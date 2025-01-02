#[cfg(feature = "embassy")]
use super::future::EventFuture;
use super::{Event, Instance};
#[cfg(feature = "embassy")]
use crate::mode::Async;
use crate::{
    clock::peripheral::PeripheralInterrupt,
    mode::{Blocking, Mode},
};
use core::marker::PhantomData;
#[cfg(feature = "embassy")]
use enumset::EnumSet;

use super::types::{CenterAlignedMode, CountDirection};

use fugit::MicrosDurationU32;

/// 计数器
///
/// 使用向上计数模式
pub struct Counter<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

impl<'d, T: Instance, M: Mode> Counter<'d, T, M> {
    pub(super) fn new() -> Self {
        T::enable_auto_reload_buff(false);
        T::enable_single_mode(false);
        T::set_cms(CenterAlignedMode::EdgeAligned);
        T::set_dir(CountDirection::Up);

        if M::is_async() {
            T::id().enable_interrupt();
        }

        Counter {
            _t: PhantomData,
            _m: PhantomData,
        }
    }

    /// 获取当前定时器的计数频率
    #[inline]
    pub fn get_freq(&self) -> u32 {
        T::counter_frequency()
    }

    fn start_us(&mut self, us: u64) {
        let (div, arr) = T::micros_to_compute_with_rep(us);
        T::stop();
        T::set_prescaler(div);
        T::set_auto_reload(arr);
        T::event_clear(Event::UIF);
        T::start();
    }

    pub fn start_ns(&mut self, nano: u64) {
        let (div, arr) = T::nanosecond_to_compute_with_rep(nano);
        T::stop();
        T::set_prescaler(div);
        T::set_auto_reload(arr);
        T::event_clear(Event::UIF);
        T::start();
    }
}

impl<'d, T: Instance, M: Mode> Drop for Counter<'d, T, M> {
    fn drop(&mut self) {
        if M::is_async() {
            T::id().disable_interrupt();
        }
    }
}

impl<'d, T: Instance> Counter<'d, T, Blocking> {
    /// 阻塞等待直到更新事件发生
    #[inline]
    pub fn delay_us_blocking(&mut self, us: u32) {
        self.start_us(us as u64);
        while !T::event_flag(Event::UIF) {}
        T::stop();
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'d, T: Instance> embedded_hal_027::blocking::delay::DelayUs<u32> for Counter<'d, T, Blocking> {
    fn delay_us(&mut self, us: u32) {
        self.delay_us_blocking(us)
    }
}

impl<'d, T: Instance> embedded_hal_027::blocking::delay::DelayMs<u32> for Counter<'d, T, Blocking> {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us_blocking(ms * 1000);
    }
}

impl<'d, T: Instance> embedded_hal_027::blocking::delay::DelayMs<u8> for Counter<'d, T, Blocking> {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_us_blocking(ms as u32 * 1000);
    }
}

impl<'d, T: Instance> embedded_hal_027::timer::CountDown for Counter<'d, T, Blocking> {
    type Time = MicrosDurationU32;
    fn start<H>(&mut self, count: H)
    where
        H: Into<Self::Time>,
    {
        self.start_us(count.into().to_micros() as u64);
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        while !T::event_flag(Event::UIF) {}
        T::stop();
        Ok(())
    }
}

/////////////////////// Async ///////////////////////

#[cfg(feature = "embassy")]
impl<'d, T: Instance> embedded_hal_async::delay::DelayNs for Counter<'d, T, Async> {
    async fn delay_ns(&mut self, ns: u32) {
        self.start_ns(ns as u64);
        let _ = EventFuture::<T>::new(EnumSet::empty() | Event::UIF).await;
    }
}
