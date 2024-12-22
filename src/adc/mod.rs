//! ADC

#[cfg(feature = "embassy")]
mod future;
mod hal;
mod pins;
mod types;

#[cfg(not(feature = "embassy"))]
mod interrupt;

#[cfg(not(feature = "embassy"))]
pub use interrupt::*;

#[cfg(feature = "embassy")]
use core::{future::Future, task::Poll};

use core::marker::PhantomData;

#[cfg(feature = "embassy")]
use crate::mcu::peripherals::ADC;
#[cfg(feature = "embassy")]
use crate::mode::Async;
use enumset::EnumSet;
#[cfg(feature = "embassy")]
use future::ChannelInputFuture;

pub use types::*;

use crate::{
    clock::peripheral::PeripheralInterrupt, macro_def::impl_sealed_peripheral_id, mode::Blocking,
};

use embassy_hal_internal::Peripheral;
pub use pins::{TemperatureChannel, VRrefChannel};

use crate::{
    clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex},
    delay::wait_for_true_timeout_block,
    mode::Mode,
};

#[cfg(feature = "embassy")]
use embassy_sync::waitqueue::AtomicWaker;

#[cfg(feature = "embassy")]
static ADC_INT_WAKER: [AtomicWaker; 1] = [AtomicWaker::new()];

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

#[derive(PartialEq)]
pub enum Id {
    ADC1 = 0,
}

impl_sealed_peripheral_id!(ADC, ADC1);

impl PeripheralIdToClockIndex for Id {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::ADC1 => PeripheralClockIndex::ADC,
        }
    }
}

impl PeripheralInterrupt for Id {
    fn interrupt(&self) -> crate::pac::interrupt {
        match *self {
            Self::ADC1 => crate::pac::interrupt::ADC_COMP,
        }
    }
}

pub struct AnyAdc<'d, T: Instance, M: Mode> {
    t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

impl<'d, T: Instance, M: Mode> AnyAdc<'d, T, M> {
    pub fn new(
        _adc: impl Peripheral<P = T>,
        config: Config,
        channel_config: ChannelConfig,
        channels: &[AdcChannel],
    ) -> Result<Self, Error> {
        T::id().clock().reset();
        T::id().clock().open();

        T::stop();

        Self::new_inner(config, channel_config, channels)?;

        T::enable();

        // 异步方式需要打开外设中断
        if M::is_async() {
            T::id().enable_interrupt();
        }

        Ok(Self {
            t: PhantomData,
            _m: PhantomData,
        })
    }

    /// 校准 adc
    pub fn calibration(config: CalibrationConfig, timeout: usize) -> Result<(), Error> {
        T::set_calibration_content(config.content);
        T::set_calibration_sample_time(config.sample_time);
        T::calibration_start();

        let block = T::block();
        wait_for_true_timeout_block(timeout, || {
            block.ccsr.read().calon().bit_is_clear() && block.ccsr.read().calfail().bit_is_clear()
        })
        .map_err(|_| Error::Calibrate)?;

        Ok(())
    }

    #[inline]
    pub fn id(&self) -> Id {
        T::id()
    }

    #[inline]
    pub fn start(&self) {
        for e in EnumSet::all() {
            T::event_clear(e);
        }

        T::start();
    }

    #[inline]
    pub fn stop(&self) {
        T::stop()
    }

    fn new_inner(
        config: Config,
        channel_config: ChannelConfig,
        channels: &[AdcChannel],
    ) -> Result<(), Error> {
        const CALIBRATE_TIMEOUT: usize = 1000000;
        T::disable();
        // 设置时钟
        T::set_clock_mode(config.clock);
        T::set_resolution(config.resolution);
        T::set_sample_cycle(config.sample_cycle);
        // 上电后硬件会自动校准一次
        if config.calibration {
            // 必须先校准再开启时钟
            Self::calibration(Default::default(), CALIBRATE_TIMEOUT)?
        }
        T::align(config.align);

        Self::channel_config(channel_config);

        // 使能通道
        for channel in channels {
            T::channel_enable(*channel, true)
        }

        Ok(())
    }

    pub fn channel_enable(&self, channels: &[impl AnalogPin<T>]) {
        for channel in channels {
            channel.as_anlog();
            T::channel_enable(channel.channel(), true);
        }
    }

    pub fn event_config(&mut self, event: Event, en: bool) {
        T::event_config(event, en);
    }

    pub fn event_clear(&mut self, event: Event) {
        T::event_clear(event);
    }

    pub fn event_flag(&self, event: Event) -> bool {
        T::event_flag(event)
    }

    pub fn set_watchdog(config: Option<WatchDogConfig>) {
        if let Some(config) = config {
            T::set_watch_dog_threshold(config.high, config.low)
        }
    }

    fn channel_config(config: ChannelConfig) {
        T::conversion_mode(config.mode);
        T::set_scan_dir(config.scan_dir);
        T::set_overwrite(config.over_write);
        T::trigle_signal(config.signal);
        T::set_wait(config.wait);
    }

    #[cfg(not(feature = "embassy"))]
    pub fn on_interrupt(
        &mut self,
        events: EnumSet<Event>,
        callback: alloc::boxed::Box<dyn Fn(u16)>,
    ) {
        crate::interrupt::register(
            #[allow(static_mut_refs)]
            unsafe {
                &mut CLOSURE
            },
            alloc::boxed::Box::new(move || {
                callback(T::data_read());
                for e in events {
                    T::event_flag(e);
                }
            }),
        );
        for e in events {
            self.event_config(e, true);
        }
    }
}

impl<'d, T: Instance, M: Mode> Drop for AnyAdc<'d, T, M> {
    fn drop(&mut self) {
        if M::is_async() {
            T::id().disable_interrupt();
        }
    }
}

impl<'d, T: Instance> AnyAdc<'d, T, Blocking> {
    pub fn read_block(&self, timeout: usize) -> Result<u16, Error> {
        wait_for_true_timeout_block(timeout, || T::event_flag(Event::EOC))
            .map_err(|_| Error::Timeout)?;
        Ok(T::data_read())
    }
}

#[cfg(feature = "embassy")]
impl<'d, T: Instance> AnyAdc<'d, T, Async> {
    pub async fn read(&self, channel: impl AnalogPin<T>) -> u16 {
        ChannelInputFuture::<T>::new_with_channel(channel.channel()).await
    }
}

#[derive(Clone, Copy)]
pub struct CalibrationConfig {
    content: CalibrationSelect,
    sample_time: CalibrationSampleTime,
}

impl CalibrationConfig {
    pub fn new(content: CalibrationSelect, sample_time: CalibrationSampleTime) -> Self {
        Self {
            content,
            sample_time,
        }
    }
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            content: CalibrationSelect::OffsetLinearity,
            sample_time: CalibrationSampleTime::Cycle_8,
        }
    }
}

pub struct WatchDogConfig {
    high: u16,
    low: u16,
    // interrupt: bool,
}

pub struct Config {
    /// 是否初始化前是否开始校验
    calibration: bool,
    /// 采样周期
    sample_cycle: SampleCycles,
    /// adc 精度
    resolution: Resolution,
    /// 数据对齐
    align: Align,
    ///  adc 时钟源
    clock: ClockMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            calibration: true,
            sample_cycle: SampleCycles::Cycle_3_5,
            resolution: Resolution::Bit12,
            align: Align::Right,
            clock: ClockMode::PCLK,
        }
    }
}

impl Config {
    pub fn new(
        calibration: bool,
        sample_cycle: SampleCycles,
        resolution: Resolution,
        align: Align,
        clock: ClockMode,
    ) -> Self {
        Self {
            calibration,
            sample_cycle,
            resolution,
            align,
            clock,
        }
    }

    pub fn sample(self, sample: SampleCycles) -> Self {
        Self {
            sample_cycle: sample,
            ..self
        }
    }

    pub fn calibration(self, calibration: bool) -> Self {
        Self {
            calibration,
            ..self
        }
    }

    pub fn resolution(self, resolution: Resolution) -> Self {
        Self { resolution, ..self }
    }

    pub fn align(self, align: Align) -> Self {
        Self { align, ..self }
    }

    pub fn clock(self, clock: ClockMode) -> Self {
        Self { clock, ..self }
    }
}

pub struct ChannelConfig {
    /// 转换模式
    mode: ConversionMode,
    /// 扫描方向
    scan_dir: ScanDir,
    /// 过写使能
    over_write: bool,
    /// 等待读取再开始下一个通道转换
    wait: bool,
    /// 触发信号类型
    signal: TrigleSignal,
}

impl ChannelConfig {
    pub fn mode(self, mode: ConversionMode) -> Self {
        Self { mode, ..self }
    }

    pub fn singal(self, signal: TrigleSignal) -> Self {
        Self { signal, ..self }
    }

    pub fn scan_dir(self, scan_dir: ScanDir) -> Self {
        Self { scan_dir, ..self }
    }

    pub fn wait(self, wait: bool) -> Self {
        Self { wait, ..self }
    }

    pub fn over_write(self, over_write: bool) -> Self {
        Self { over_write, ..self }
    }
    /// 多通道配置读取推荐配置
    /// 连续转换/向上扫描/不过写/软件触发
    pub fn new_multiple_channel_perferred() -> Self {
        Self {
            mode: ConversionMode::Continuous,
            scan_dir: ScanDir::Up,
            over_write: false,
            wait: false,
            signal: TrigleSignal::Soft,
        }
    }

    /// 单通道读取推荐配置
    /// 连续转换/向上扫描/过写/软件触发
    pub fn new_exclusive_perferred() -> Self {
        Self {
            mode: ConversionMode::Continuous,
            scan_dir: ScanDir::Up,
            over_write: true,
            wait: true,
            signal: TrigleSignal::Soft,
        }
    }

    /// 单次扫描模式
    pub fn new_exclusive_single() -> Self {
        Self {
            mode: ConversionMode::Single,
            scan_dir: ScanDir::Up,
            over_write: false,
            wait: true,
            signal: TrigleSignal::Soft,
        }
    }
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            mode: ConversionMode::Continuous,
            scan_dir: ScanDir::Up,
            over_write: true,
            wait: true,
            signal: TrigleSignal::Soft,
        }
    }
}

pub trait AnalogPin<T: Instance> {
    fn channel(&self) -> AdcChannel;
    fn as_anlog(&self);
}

pub fn temperature(dr: u16) -> f32 {
    const TS_CAL1_ADDR: u32 = 0x1fff_0f14;
    const TS_CAL2_ADDR: u32 = 0x1fff_0f18;

    let ts_cal2 = unsafe { core::ptr::read(TS_CAL2_ADDR as *const u32) } as f32;
    let ts_cal1 = unsafe { core::ptr::read(TS_CAL1_ADDR as *const u32) } as f32;
    // let temp_k = (85.0 - 30.0) / (ts_cal2 - ts_cal1);
    // temp_k * (dr as f32 - ts_cal1) + ts_cal1
    // dr as f32 / 4095.0 * 3.3
    ((85.0 - 30.0) * (dr as f32 - ts_cal1) / (ts_cal2 - ts_cal1)) + 30.0
}

pub fn vrefence_internal(dr: u16) -> f32 {
    // dr as f32 / 4095.0 * 3.3
    4095.0 * 1.2 / dr as f32
}
