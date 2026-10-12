#[cfg(feature = "embassy")]
use super::future::EventFuture;
use super::{Event, Instance};
use super::{WithCenterAlign, WithRepetition};
#[cfg(feature = "embassy")]
use crate::mode::Async;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;
#[cfg(feature = "embassy")]
use enumset::EnumSet;
use fugit::MicrosDurationU32;

/// 统一计数器 / 阻塞延时
///
/// TIM1 / TIM3 / TIM14 / TIM16 共用同一套实现；只有部分定时器具备的
/// 能力（重复计数、中心对齐）通过 capability trait 在编译期门控。
pub struct Counter<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

impl<'d, T: Instance, M: Mode> Counter<'d, T, M> {
    pub(super) fn new() -> Self {
        T::enable_auto_reload_buff(false);

        if M::is_async() {
            // TIM1 有两个中断向量，这里会全部 unmask
            T::id().enable_interrupts();
        }

        Counter {
            _t: PhantomData,
            _m: PhantomData,
        }
    }

    /// 返回定时器外设时钟
    #[inline]
    pub fn get_timer_clock(&self) -> u32 {
        T::get_time_pclk()
    }

    /// 获取当前计数频率
    #[inline]
    pub fn get_freq(&self) -> u32 {
        T::counter_frequency()
    }

    /// 读取当前计数值
    #[inline]
    pub fn now(&self) -> u16 {
        T::get_cnt()
    }

    /// 停止计数
    #[inline]
    pub fn stop(&self) {
        T::stop();
    }

    /// 由微秒换算 (PSC, ARR)。不依赖重复计数器，因此所有定时器通用。
    fn us_to_psc_arr(us: u64) -> (u16, u16) {
        let ticks = us * T::get_time_pclk() as u64 / 1_000_000;
        let psc = (ticks / 0x1_0000).min(0xFFFF) as u16;
        let arr = (ticks / (psc as u64 + 1)).min(0xFFFF) as u16;
        (psc, arr)
    }

    fn ns_to_psc_arr(ns: u64) -> (u16, u16) {
        let ticks = ns * T::get_time_pclk() as u64 / 1_000_000_000;
        let psc = (ticks / 0x1_0000).min(0xFFFF) as u16;
        let arr = (ticks / (psc as u64 + 1)).min(0xFFFF) as u16;
        (psc, arr)
    }

    /// 以给定时长启动一次计数（不阻塞）
    fn start_us(&mut self, us: u64) {
        let (psc, arr) = Self::us_to_psc_arr(us);
        T::stop();
        T::set_prescaler(psc);
        T::set_auto_reload(arr);
        T::event_clear(Event::UIF);
        T::start();
    }

    /// 以纳秒启动一次计数（不阻塞）
    pub fn start_ns(&mut self, nano: u64) {
        let (psc, arr) = Self::ns_to_psc_arr(nano);
        T::stop();
        T::set_prescaler(psc);
        T::set_auto_reload(arr);
        T::event_clear(Event::UIF);
        T::start();
    }

    /// 阻塞等待更新事件
    #[inline]
    pub fn wait(&self) {
        while !T::event_flag(Event::UIF) {}
        T::event_clear(Event::UIF);
    }
}

//////////////// 仅具备重复计数器的定时器（TIM1 / TIM16） ////////////////

impl<'d, T: Instance + WithRepetition, M: Mode> Counter<'d, T, M> {
    /// 设置重复计数值（更新事件在重复 N+1 次后产生）
    #[inline]
    pub fn set_repetition(&self, rep: u8) {
        T::set_repetition(rep);
    }
}

//////////////// 仅具备中心对齐/向下计数的定时器（TIM1 / TIM3） ////////////////

impl<'d, T: Instance + WithCenterAlign, M: Mode> Counter<'d, T, M> {
    /// 设置中心对齐模式
    #[inline]
    pub fn set_center_aligned(&self, mode: super::CenterAlignedMode) {
        T::set_cms(mode);
    }

    /// 设置计数方向
    #[inline]
    pub fn set_direction(&self, dir: super::CountDirection) {
        T::set_dir(dir);
    }
}

impl<'d, T: Instance, M: Mode> Drop for Counter<'d, T, M> {
    fn drop(&mut self) {
        if M::is_async() {
            T::id().disable_interrupts();
        }
    }
}

impl<'d, T: Instance> Counter<'d, T, Blocking> {
    /// 阻塞延时（微秒）
    #[inline]
    pub fn delay_us_blocking(&mut self, us: u32) {
        self.start_us(us as u64);
        while !T::event_flag(Event::UIF) {}
        T::event_clear(Event::UIF);
        T::stop();
    }

    /// 阻塞延时（毫秒）
    #[inline]
    pub fn delay_ms_blocking(&mut self, ms: u32) {
        self.delay_us_blocking(ms.saturating_mul(1000));
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'d, T: Instance> embedded_hal_027::blocking::delay::DelayUs<u32>
    for Counter<'d, T, Blocking>
{
    fn delay_us(&mut self, us: u32) {
        self.delay_us_blocking(us)
    }
}

impl<'d, T: Instance> embedded_hal_027::blocking::delay::DelayMs<u32>
    for Counter<'d, T, Blocking>
{
    fn delay_ms(&mut self, ms: u32) {
        self.delay_ms_blocking(ms)
    }
}

impl<'d, T: Instance> embedded_hal_027::blocking::delay::DelayMs<u8> for Counter<'d, T, Blocking> {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms_blocking(ms as u32)
    }
}

impl<'d, T: Instance> embedded_hal_async::delay::DelayNs for Counter<'d, T, Async> {
    async fn delay_ns(&mut self, ns: u32) {
        self.start_ns(ns as u64);
        let _ = EventFuture::<T>::new(EnumSet::empty() | Event::UIF).await;
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
        T::event_clear(Event::UIF);
        T::stop();
        Ok(())
    }
}
