mod hal;
mod types;

use core::{future::Future, marker::PhantomData, task::Poll};

use crate::{
    clock::peripheral::PeripheralInterrupt,
    macro_def::impl_sealed_peripheral_id,
    mcu::peripherals::RTC,
    mode::{Async, Blocking},
    pac::interrupt,
    pwr::pwr,
};

use embassy_hal_internal::Peripheral;
use enumset::{EnumSet, EnumSetType};
use types::*;

use crate::{
    clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex},
    delay::wait_for_true_timeout_block,
    mode::Mode,
};

use embassy_sync::waitqueue::AtomicWaker;

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
            .for_each(|event| T::enable_interrupt(event, false));

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
}

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
            T::enable_interrupt(event, true);
        });
        T::id().enable_interrupt();
        WakeFuture::<T>::new(event).await
    }

    pub async fn wait_second(&self) {
        let _ = T::enable_config();

        // T::disable_config();
        let event = EnumSet::empty() | EventKind::Second;
        // 开启中断使能
        event.iter().for_each(|event| {
            T::clear_interrupt(event);
            T::enable_interrupt(event, true);
        });
        T::id().enable_interrupt();
        WakeFuture::<T>::new(event).await
    }
}

#[derive(EnumSetType, Debug)]
pub enum EventKind {
    Alarm,
    Second,
    OverFlow,
}

static WAKER: [AtomicWaker; 1] = [AtomicWaker::new()];

pub struct WakeFuture<T: Instance> {
    _t: PhantomData<T>,
    event: EnumSet<EventKind>,
}

impl<T: Instance> WakeFuture<T> {
    pub fn new(event: EnumSet<EventKind>) -> Self {
        Self {
            _t: PhantomData,
            event,
        }
    }

    #[inline]
    fn on_interrupt() {
        EnumSet::all().iter().for_each(|event| {
            if T::is_interrupt(event) && T::is_enable_interrupt(event) {
                T::enable_interrupt(event, false);
            }
        });
        WAKER[T::id() as usize].wake()
    }
}

impl<T: Instance> Future for WakeFuture<T> {
    type Output = ();
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        WAKER[T::id() as usize].register(cx.waker());

        let mut interrupt = false;

        self.event.iter().for_each(|event| {
            if T::is_interrupt(event) {
                interrupt = true;
                T::clear_interrupt(event);
            }
        });

        if interrupt {
            T::disable_config();
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl<T: Instance> Drop for WakeFuture<T> {
    fn drop(&mut self) {}
}

#[interrupt]
fn RTC() {
    critical_section::with(|_cs| WakeFuture::<RTC>::on_interrupt())
}
