#[cfg(feature = "embassy")]
mod future;
mod hal;
mod types;
#[cfg(feature = "embassy")]
use crate::mode::Async;
use crate::{
    clock::peripheral::PeripheralInterrupt,
    macro_def::impl_sealed_peripheral_id,
    // mcu::peripherals::RTC,
    mode::Blocking,

    pwr::pwr,
};
use core::marker::PhantomData;
use embassy_hal_internal::Peripheral;
use enumset::EnumSet;
pub use types::*;

use crate::{
    clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex},
    delay::wait_for_true_timeout_block,
    mode::Mode,
};

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

#[derive(PartialEq)]
pub(crate) enum Id {
    Rtc1 = 0,
}

// 根据ID匹配外设实体
impl_sealed_peripheral_id!(RTC, Rtc1);

impl PeripheralIdToClockIndex for Id {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::Rtc1 => PeripheralClockIndex::RTCAPB,
        }
    }
}

impl PeripheralInterrupt for Id {
    fn interrupt(&self) -> crate::pac::interrupt {
        match *self {
            Self::Rtc1 => crate::pac::interrupt::RTC,
        }
    }
}

pub struct Config {
    clock: RtcClock,
    load: Option<u32>,
}

impl Config {
    pub fn new(clock: RtcClock, load: Option<u32>) -> Self {
        Self { clock, load }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clock: RtcClock::LSI,
            load: None,
        }
    }
}

pub struct AnyRtc<'d, T: Instance, M: Mode> {
    t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

impl<'d, T: Instance, M: Mode> AnyRtc<'d, T, M> {
    pub fn new(_rtc: impl Peripheral<P = T>, config: Config) -> Result<Self, Error> {
        // 开启时钟
        T::id().clock().open();
        // T::id().reset();

        Self::new_inner(config)?;

        Ok(Self {
            t: PhantomData,
            _m: PhantomData,
        })
    }

    fn new_inner(config: Config) -> Result<(), Error> {
        pwr::rtc_unlock(true);
        T::set_clock(config.clock)?;

        let _ = T::enable_config();

        // 关闭所有中断
        EnumSet::all()
            .iter()
            .for_each(|event| T::event_config(event, false));

        // 重新写入计数值
        if let Some(load) = config.load {
            T::set_counter(load);
        }

        T::disable_config();

        Ok(())
    }

    #[inline]
    pub fn read(&self) -> u32 {
        T::get_counter()
    }
}

impl<'d, T: Instance> AnyRtc<'d, T, Blocking> {
    #[inline]
    pub fn wait_block(&self, second: u32) {
        let br = self.read() + second;
        while self.read() < br {}
    }

    pub fn wait_alarm(&self, after: u32) {
        let br = self.read() + after - 1;
        let _ = T::enable_config();
        T::set_alarm(br);

        T::clear_interrupt(EventKind::Alarm);

        while !T::event_flag(EventKind::Alarm) {}
    }

    pub fn wait_second(&self) {
        T::clear_interrupt(EventKind::Second);
        while !T::event_flag(EventKind::Second) {}
    }
}

#[cfg(feature = "embassy")]
impl<'d, T: Instance> AnyRtc<'d, T, Async> {
    #[inline]
    pub async fn wait_alarm(&self, after: u32) {
        let br = self.read() + after - 1;
        let _ = T::enable_config();
        T::set_alarm(br);
        // T::disable_config();
        let event = EnumSet::empty() | EventKind::Alarm;
        // 开启中断使能
        event.iter().for_each(|event| {
            T::clear_interrupt(event);
            T::event_config(event, true);
        });
        T::id().enable_interrupt();
        future::WakeFuture::<T>::new(event).await
    }

    pub async fn wait_second(&self) {
        let _ = T::enable_config();

        // T::disable_config();
        let event = EnumSet::empty() | EventKind::Second;
        // 开启中断使能
        event.iter().for_each(|event| {
            T::clear_interrupt(event);
            T::event_config(event, true);
        });
        T::id().enable_interrupt();
        future::WakeFuture::<T>::new(event).await
    }
}
