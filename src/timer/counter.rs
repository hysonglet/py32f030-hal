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

/// Unified counter / blocking delay.
///
/// TIM1 / TIM3 / TIM14 / TIM16 share a single implementation; capabilities that
/// only some timers have (repetition counter, center-aligned mode) are gated at
/// compile time through capability traits.
pub struct Counter<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

impl<'d, T: Instance, M: Mode> Counter<'d, T, M> {
    pub(super) fn new() -> Self {
        T::enable_auto_reload_buff(false);

        if M::is_async() {
            // TIM1 has two interrupt vectors; this unmarks both of them.
            T::id().enable_interrupts();
        }

        Counter {
            _t: PhantomData,
            _m: PhantomData,
        }
    }

    /// Returns the timer peripheral clock.
    #[inline]
    pub fn get_timer_clock(&self) -> u32 {
        T::get_time_pclk()
    }

    /// Get the current counting frequency.
    #[inline]
    pub fn get_freq(&self) -> u32 {
        T::counter_frequency()
    }

    /// Read the current counter value.
    #[inline]
    pub fn now(&self) -> u16 {
        T::get_cnt()
    }

    /// Stop counting.
    #[inline]
    pub fn stop(&self) {
        T::stop();
    }

    /// Convert microseconds into (PSC, ARR). Independent of the repetition counter,
    /// so it is common to all timers.
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

    /// Start a single count for the given duration (non-blocking).
    fn start_us(&mut self, us: u64) {
        let (psc, arr) = Self::us_to_psc_arr(us);
        T::stop();
        T::set_prescaler(psc);
        T::set_auto_reload(arr);
        T::event_clear(Event::UIF);
        T::start();
    }

    /// Start a single count for the given nanoseconds (non-blocking).
    pub fn start_ns(&mut self, nano: u64) {
        let (psc, arr) = Self::ns_to_psc_arr(nano);
        T::stop();
        T::set_prescaler(psc);
        T::set_auto_reload(arr);
        T::event_clear(Event::UIF);
        T::start();
    }

    /// Block until the update event.
    #[inline]
    pub fn wait(&self) {
        while !T::event_flag(Event::UIF) {}
        T::event_clear(Event::UIF);
    }
}


impl<'d, T: Instance + WithRepetition, M: Mode> Counter<'d, T, M> {
    /// Set the repetition counter (the update event is generated after N+1 repetitions).
    #[inline]
    pub fn set_repetition(&self, rep: u8) {
        T::set_repetition(rep);
    }
}


impl<'d, T: Instance + WithCenterAlign, M: Mode> Counter<'d, T, M> {
    /// Set the center-aligned mode.
    #[inline]
    pub fn set_center_aligned(&self, mode: super::CenterAlignedMode) {
        T::set_cms(mode);
    }

    /// Set the count direction.
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
    /// Blocking delay (microseconds).
    #[inline]
    pub fn delay_us_blocking(&mut self, us: u32) {
        self.start_us(us as u64);
        while !T::event_flag(Event::UIF) {}
        T::event_clear(Event::UIF);
        T::stop();
    }

    /// Blocking delay (milliseconds).
    #[inline]
    pub fn delay_ms_blocking(&mut self, ms: u32) {
        self.delay_us_blocking(ms.saturating_mul(1000));
    }
}


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
