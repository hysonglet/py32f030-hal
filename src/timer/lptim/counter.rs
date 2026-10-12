use super::{ClockSource, Event, Instance, LpTimerConfig};
use core::marker::PhantomData;
use fugit::MicrosDurationU32;

/// 低功耗定时器计数器
///
/// LPTIM 为 16 位向上计数器，仅支持 ARR 自动重装载匹配中断（ARRM）。
/// 可用作低功耗延时/唤醒源（Sleep/Stop 下由 LSI/LSE 驱动仍可运行）。
///
/// 注意：当前 PAC 的 `Interrupt` 枚举缺少 `LPTIM1` 中断向量（硬件向量 17），
/// 因此中断/ embassy 异步路径暂不可用；本结构提供完整的**阻塞轮询** API。
pub struct Counter<'d, T: Instance> {
    _t: PhantomData<&'d T>,
    config: LpTimerConfig,
}

impl<'d, T: Instance> Counter<'d, T> {
    pub(super) fn new(config: LpTimerConfig) -> Self {
        // CFGR/IER 仅在 LPTIM 禁用时可写
        T::stop();
        T::set_clock_source(config.clock_source);
        T::set_prescaler(config.prescaler);
        T::set_preload(config.preload);

        Counter {
            _t: PhantomData,
            config,
        }
    }

    /// 内核时钟频率（Hz）：PCLK / LSI / LSE，取决于配置
    pub fn kernel_clock_hz(&self) -> u32 {
        match self.config.clock_source {
            ClockSource::Pclk => crate::clock::timer_pclk(),
            // 与 crate::clock 中 LSI/LSE 频率常量保持一致（均为 32768）
            ClockSource::Lsi => 32_768,
            ClockSource::Lse => 32_768,
        }
    }

    /// 计数时钟频率（Hz）：内核时钟经预分频后计数器递增的频率
    pub fn get_freq(&self) -> u32 {
        self.kernel_clock_hz() / self.config.prescaler.divisor()
    }

    /// 写自动重装载值（ARR）
    pub fn set_auto_reload(&self, arr: u16) {
        T::set_auto_reload(arr);
    }

    /// 读当前计数值
    pub fn now(&self) -> u16 {
        T::counter()
    }

    /// 使能内核（连续计数需先 `set_auto_reload`）
    pub fn enable(&self) {
        T::enable_kernel(true);
    }

    /// 停止计数
    pub fn stop(&self) {
        T::stop();
    }

    /// 启动连续计数（到 ARR 后自动重载并持续运行）
    pub fn start_continuous(&self, arr: u16) {
        T::stop();
        T::set_auto_reload(arr);
        T::event_clear(Event::AutoreloadMatch);
        T::start_continuous();
    }

    /// 启动单次计数（到 ARR 后停止并产生 ARRM，可用于唤醒）
    pub fn start_single(&self, arr: u16) {
        T::stop();
        T::set_auto_reload(arr);
        T::event_clear(Event::AutoreloadMatch);
        T::start_single();
    }

    /// 阻塞等待一次自动重装载匹配（ARRM）
    pub fn wait(&self) {
        while !T::event_flag(Event::AutoreloadMatch) {}
        T::event_clear(Event::AutoreloadMatch);
    }

    /// 阻塞延时（以计数时钟 tick 为单位，单次最多 0xFFFF）
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

////////////////////////////////////////////////////////////////////////////////

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
