use super::{Event, Instance};
use crate::{
    mode::{Blocking, Mode},
    timer::advanced_timer::{CenterAlignedMode, CountDirection},
};
use core::marker::PhantomData;
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
        T::set_dir(CountDirection::Down);

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
}

impl<'d, T: Instance> Counter<'d, T, Blocking> {
    fn start_us(&mut self, us: u64) {
        let (div, rep, arr) = T::micros_to_compute_with_rep(us);
        T::stop();
        T::set_prescaler(div);
        T::set_repetition(rep);
        T::set_auto_reload(arr);
        T::event_clear(Event::UIF);
        T::start();
    }

    /// 阻塞等待直到更新事件发生
    #[inline]
    pub fn delay_us_blocking(&mut self, us: u32) {
        self.start_us(us as u64);
        while T::event_flag(Event::UIF) == false {}
        T::stop();
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'d, T: Instance> embedded_hal::blocking::delay::DelayUs<u32> for Counter<'d, T, Blocking> {
    fn delay_us(&mut self, us: u32) {
        self.delay_us_blocking(us)
    }
}

impl<'d, T: Instance> embedded_hal::blocking::delay::DelayMs<u32> for Counter<'d, T, Blocking> {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us_blocking(ms * 1000);
    }
}

impl<'d, T: Instance> embedded_hal::timer::CountDown for Counter<'d, T, Blocking> {
    type Time = MicrosDurationU32;
    fn start<H>(&mut self, count: H)
    where
        H: Into<Self::Time>,
    {
        self.start_us(count.into().to_micros() as u64);
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        while T::event_flag(Event::UIF) == false {}
        T::stop();
        Ok(())
    }
}
