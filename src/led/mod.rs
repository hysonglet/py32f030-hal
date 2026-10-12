//! 段式 LED 显示驱动控制器（LED）
//!
//! 该外设用于驱动多 COM 复用的段式 LCD/LED 屏：通过 `dr0`..`dr3` 设置每一条
//! COM 线上各段（A–G、DP 共 8 段）的亮灭，由内部扫描电路自动循环扫描。
//! 参考手册：LED 控制器章节。

mod hal;

use crate::clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex};
use crate::impl_sealed_peripheral_id;
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};

/// LED 实例索引（仅 LED1）
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Id {
    Led1 = 0,
}

impl PeripheralIdToClockIndex for Id {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::Led1 => PeripheralClockIndex::LED,
        }
    }
}

/// 亮度控制（CR.EHS，2 位）
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum LightControl {
    /// 最暗
    #[default]
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    /// 最亮
    Level3 = 3,
}

/// LED 配置
#[derive(Clone, Copy, Debug)]
pub struct LedConfig {
    /// 预分频器（0–255）：扫描时钟 = LED 时钟 / (pr + 1)
    pub prescaler: u8,
    /// COM 线数量（1–4）
    pub com_count: u8,
    /// 显示时序 t1
    pub t1: u8,
    /// 显示时序 t2
    pub t2: u8,
    /// 亮度控制
    pub light_control: LightControl,
    /// 是否使能扫描完成中断
    pub interrupt: bool,
}

impl Default for LedConfig {
    fn default() -> Self {
        Self {
            prescaler: 0,
            com_count: 4,
            t1: 0,
            t2: 0,
            light_control: LightControl::Level3,
            interrupt: false,
        }
    }
}

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

impl_sealed_peripheral_id!(LED, Led1);

/// LED 显示控制器句柄
pub struct AnyLed<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> AnyLed<'d, T> {
    /// 新建并按配置开启扫描
    pub fn new(_led: impl Peripheral<P = T> + 'd, config: LedConfig) -> Self {
        into_ref!(_led);

        T::id().clock().open();
        Self::apply_config(config);

        Self { _t: PhantomData }
    }

    fn apply_config(config: LedConfig) {
        T::set_prescaler(config.prescaler);
        T::set_com_count(config.com_count);
        T::set_timing(config.t1, config.t2);
        T::set_light_control(config.light_control);
        T::set_interrupt(config.interrupt);
    }

    /// 重新配置
    #[inline]
    pub fn reconfig(&self, config: LedConfig) {
        Self::apply_config(config);
    }

    /// 开启 / 关闭显示扫描
    #[inline]
    pub fn enable(&self, en: bool) {
        T::enable(en);
    }

    /// 设置预分频器
    #[inline]
    pub fn set_prescaler(&self, pr: u8) {
        T::set_prescaler(pr);
    }

    /// 设置 COM 数量（1–4）
    #[inline]
    pub fn set_com_count(&self, count: u8) {
        T::set_com_count(count);
    }

    /// 设置显示时序 t1 / t2
    #[inline]
    pub fn set_timing(&self, t1: u8, t2: u8) {
        T::set_timing(t1, t2);
    }

    /// 设置亮度控制
    #[inline]
    pub fn set_light_control(&self, lc: LightControl) {
        T::set_light_control(lc);
    }

    /// 写入某条 COM 线的 8 段显示数据
    ///
    /// `seg` 的 bit0–bit7 分别对应 A、B、C、D、E、F、G、DP 段。
    #[inline]
    pub fn set_display_data(&self, com: u8, seg: u32) {
        T::set_display_data(com, seg);
    }

    /// 使能扫描完成中断
    #[inline]
    pub fn enable_interrupt(&self, en: bool) {
        T::set_interrupt(en);
    }

    /// 中断是否挂起
    #[inline]
    pub fn is_interrupt_pending(&self) -> bool {
        T::is_interrupt_pending()
    }

    /// 清除中断标志
    #[inline]
    pub fn clear_interrupt(&self) {
        T::clear_interrupt();
    }

    /// 停止扫描并关闭外设时钟
    pub fn deinit(self) {
        T::enable(false);
        T::id().clock().close();
    }
}

impl<'d, T: Instance> Drop for AnyLed<'d, T> {
    fn drop(&mut self) {
        T::enable(false);
        T::id().clock().close();
    }
}
