//! 模拟比较器（COMP1 / COMP2）
//!
//! 说明：本芯片 COMP 的中断与 ADC 共用 `ADC_COMP` 向量（且该向量已被 adc 模块占用），
//! 因此本模块仅提供**轮询**方式的输出读取，不提供中断/异步接口（同 LPTIM 的处理）。

mod hal;
mod types;

pub use types::*;

use crate::clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + hal::sealed::CompInstance + 'static + Send {}

/// 比较器索引
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Id {
    COMP1 = 0,
    COMP2 = 1,
}

impl PeripheralIdToClockIndex for Id {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::COMP1 => PeripheralClockIndex::COMP1,
            Self::COMP2 => PeripheralClockIndex::COMP2,
        }
    }
}

hal::sealed::impl_comp_instance!(COMP1, COMP1, COMP1, has_hyst);
hal::sealed::impl_comp_instance!(COMP2, COMP2, COMP2, no_hyst);

/// 比较器句柄
pub struct AnyComp<'d, T: Instance> {
    _t: PhantomData<&'d T>,
    inner: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> AnyComp<'d, T> {
    /// 新建并配置比较器（默认关闭，需调用 `enable()`）
    pub fn new(comp: impl Peripheral<P = T> + 'd, config: Config) -> Self {
        into_ref!(comp);

        T::id().clock().open();
        T::config(config);

        Self {
            _t: PhantomData,
            inner: comp,
        }
    }

    /// 使能比较器
    #[inline]
    pub fn enable(&self) {
        T::enable(true);
    }

    /// 关闭比较器
    #[inline]
    pub fn disable(&self) {
        T::enable(false);
    }

    /// 读取比较器输出电平
    ///
    /// true 表示同相输入高于反相输入（极性反相时含义相反）
    #[inline]
    pub fn output(&self) -> bool {
        T::output()
    }

    /// 阻塞等待输出变为指定电平
    #[inline]
    pub fn wait_for(&self, level: bool) {
        while T::output() != level {}
    }

    /// 重新配置（会先关闭比较器）
    #[inline]
    pub fn reconfig(&self, config: Config) {
        T::config(config);
    }

    #[inline]
    pub fn set_hysteresis(&self, hyst: Hysteresis) {
        T::set_hysteresis(hyst);
    }

    #[inline]
    pub fn set_power_mode(&self, mode: PowerMode) {
        T::set_power_mode(mode);
    }

    #[inline]
    pub fn set_polarity(&self, polarity: Polarity) {
        T::set_polarity(polarity);
    }

    #[inline]
    pub fn set_window_mode(&self, en: bool) {
        T::set_window_mode(en);
    }

    /// 锁定配置；锁定后 CSR 变为只读，直到下次复位
    #[inline]
    pub fn lock(&self) {
        T::lock();
    }

    /// 配置是否已被锁定
    #[inline]
    pub fn is_locked(&self) -> bool {
        T::is_locked()
    }

    /// 设置数字滤波
    #[inline]
    pub fn set_filter(&self, en: bool, count: u8) {
        T::set_filter(en, count);
    }

    /// 释放：关闭比较器并关闭外设时钟
    pub fn deinit(self) {
        T::enable(false);
        T::id().clock().close();
    }
}

impl<'d, T: Instance> Drop for AnyComp<'d, T> {
    fn drop(&mut self) {
        T::enable(false);
    }
}

// 保持外设引用存活
impl<'d, T: Instance> core::ops::Deref for AnyComp<'d, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
