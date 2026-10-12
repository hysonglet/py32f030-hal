mod counter;
mod hal;
mod types;

pub use counter::Counter;
pub use types::*;

use core::marker::PhantomData;
use crate::clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex};
use embassy_hal_internal::{into_ref, Peripheral};

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LpTimer {
    LPTIM,
}

impl PeripheralIdToClockIndex for LpTimer {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::LPTIM => PeripheralClockIndex::LPTIM,
        }
    }
}

// 注：PAC 的 `Interrupt` 枚举缺少 `LPTIM1` 变体（硬件向量 17 存在），
// 因此本模块暂不实现 `PeripheralInterrupt` / embassy 异步，仅提供阻塞轮询 API。

macro_rules! impl_sealed_timer {
    (
        $peripheral: ident, $id: ident
    ) => {
        impl hal::sealed::Instance for crate::mcu::peripherals::$peripheral {
            fn id() -> LpTimer {
                LpTimer::$id
            }
        }
        impl Instance for crate::mcu::peripherals::$peripheral {}
    };
}

impl_sealed_timer!(LPTIM, LPTIM);

pub struct AnyLpTimer<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> AnyLpTimer<'d, T> {
    /// 新建一个低功耗定时器
    pub fn new(_timer: impl Peripheral<P = T>) -> Result<Self, Error> {
        into_ref!(_timer);

        // 开启外设时钟
        T::id().clock().open();

        Ok(Self { _t: PhantomData })
    }

    /// 转换成计数/延时模式
    pub fn as_counter(self, config: LpTimerConfig) -> Counter<'d, T> {
        Counter::new(config)
    }
}

impl<'d, T: Instance> Drop for AnyLpTimer<'d, T> {
    fn drop(&mut self) {
        T::stop();
        T::id().clock().close();
    }
}
