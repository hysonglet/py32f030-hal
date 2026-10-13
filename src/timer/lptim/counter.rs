use super::{ClockSource, Event, Instance, LpTimerConfig};
use core::marker::PhantomData;
use fugit::MicrosDurationU32;

/// Low-power timer counter.
///
/// LPTIM is a 16-bit up-counter that only supports the ARR auto-reload match
/// interrupt (ARRM). It can be used as a low-power delay / wakeup source
/// (still runs under Sleep/Stop driven by LSI/LSE).
///
/// Note: the current PAC `Interrupt` enum lacks the `LPTIM1` interrupt vector
/// (hardware vector 17), so the interrupt / embassy async path is currently
/// unavailable; this struct provides a complete **blocking polling** API.
pub struct Counter<'d, T: Instance> {
    _t: PhantomData<&'d T>,
    config: LpTimerConfig,
}

impl<'d, T: Instance> Counter<'d, T> {
    pub(super) fn new(config: LpTimerConfig) -> Self {
        // CFGR/IER are writable only when LPTIM is disabled.
        T::stop();
        T::set_clock_source(config.clock_source);
        T::set_prescaler(config.prescaler);
        T::set_preload(config.preload);

        Counter {
            _t: PhantomData,
            config,
        }
    }

    /// Core clock frequency (Hz): PCLK / LSI / LSE, depending on configuration.
    pub fn kernel_clock_hz(&self) -> u32 {
        match self.config.clock_source {
            ClockSource::Pclk => crate::clock::timer_pclk(),
            // Keep consistent with the LSI/LSE frequency constants in crate::clock (both 32768).
            ClockSource::Lsi => 32_768,
            ClockSource::Lse => 32_768,
        }
    }

    /// Count clock frequency (Hz): the rate at which the counter increments after prescaling.
    pub fn get_freq(&self) -> u32 {
        self.kernel_clock_hz() / self.config.prescaler.divisor()
    }

    /// Write the auto-reload value (ARR).
    pub fn set_auto_reload(&self, arr: u16) {
        T::set_auto_reload(arr);
    }

    /// Read the current counter value.
    pub fn now(&self) -> u16 {
        T::counter()
    }

    /// Enable the core (continuous counting requires `set_auto_reload` first).
    pub fn enable(&self) {
        T::enable_kernel(true);
    }

    /// Stop counting.
    pub fn stop(&self) {
        T::stop();
    }

    /// Start continuous counting (auto-reload and keep running after reaching ARR).
    pub fn start_continuous(&self, arr: u16) {
        T::stop();
        T::set_auto_reload(arr);
        T::event_clear(Event::AutoreloadMatch);
        T::start_continuous();
    }

    /// Start a single count (stop and generate ARRM after reaching ARR; can be used to wake up).
    pub fn start_single(&self, arr: u16) {
        T::stop();
        T::set_auto_reload(arr);
        T::event_clear(Event::AutoreloadMatch);
        T::start_single();
    }

    /// Block until the next auto-reload match (ARRM).
    pub fn wait(&self) {
        while !T::event_flag(Event::AutoreloadMatch) {}
        T::event_clear(Event::AutoreloadMatch);
    }

    /// Blocking delay (in count-clock ticks, at most 0xFFFF per call).
    pub fn delay_ticks(&self, ticks: u16) {
        if ticks == 0 {
            return;
        }
        T::stop();
        T::set_auto_reload(ticks);
        T::event_clear(Event::AutoreloadMatch);
        T::start_single();
        while !T::event_flag(Event::AutoreloadMatch) {}
        T::event_clear(Event::AutoreloadMatch);
    }

    fn delay_us_blocking(&self, us: u32) {
        let tick_hz = self.get_freq() as u64;
        let ticks = us as u64 * tick_hz / 1_000_000;
        let mut remaining = ticks;
        while remaining > 0 {
            let chunk = remaining.min(0xFFFF) as u16;
            self.delay_ticks(chunk);
            remaining -= chunk as u64;
        }
    }

    fn start_us(&self, us: u64) {
        let tick_hz = self.get_freq() as u64;
        let ticks = (us * tick_hz / 1_000_000).min(0xFFFF) as u16;
        T::stop();
        T::set_auto_reload(ticks);
        T::event_clear(Event::AutoreloadMatch);
        T::start_single();
    }
}


impl<'d, T: Instance> embedded_hal_027::blocking::delay::DelayUs<u32> for Counter<'d, T> {
    fn delay_us(&mut self, us: u32) {
        self.delay_us_blocking(us)
    }
}

impl<'d, T: Instance> embedded_hal_027::blocking::delay::DelayMs<u32> for Counter<'d, T> {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us_blocking(ms * 1000)
    }
}

impl<'d, T: Instance> embedded_hal_027::blocking::delay::DelayMs<u8> for Counter<'d, T> {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_us_blocking(ms as u32 * 1000)
    }
}

impl<'d, T: Instance> embedded_hal_027::timer::CountDown for Counter<'d, T> {
    type Time = MicrosDurationU32;

    fn start<H>(&mut self, count: H)
    where
        H: Into<Self::Time>,
    {
        self.start_us(count.into().to_micros() as u64);
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        while !T::event_flag(Event::AutoreloadMatch) {}
        T::event_clear(Event::AutoreloadMatch);
        T::stop();
        Ok(())
    }
}
