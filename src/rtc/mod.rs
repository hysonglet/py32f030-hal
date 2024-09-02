mod hal;

use core::{future::Future, marker::PhantomData, task::Poll};

use crate::{
    macro_def::impl_sealed_peripheral_id,
    mcu::peripherals::RTC,
    mode::{Async, Blocking},
    pac::interrupt,
    pwr::pwr,
};

use embassy_hal_internal::Peripheral;

use crate::{
    clock::peripheral::{PeripheralClockIndex, PeripheralEnable},
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

impl PeripheralEnable for Id {
    fn clock(&self, en: bool) {
        match *self {
            Self::Rtc1 => {
                PeripheralClockIndex::RTCAPB.clock(en);
                // patch, 开启 rtc 时钟需要开启pwr
                if en {
                    PeripheralClockIndex::PWR.clock(true);
                }
            }
        }
    }

    fn is_open(&self) -> bool {
        match *self {
            Self::Rtc1 => PeripheralClockIndex::RTCAPB.is_open(),
        }
    }

    fn reset(&self) {
        match *self {
            Self::Rtc1 => PeripheralClockIndex::RTCAPB.reset(),
        }
    }
}

/// Alarm or second output selection
pub enum PinSignal {
    /// Pin 上输出的是 alarm 信号
    AlarmPulse,
    /// Pin 上输出的是秒信号
    SecondPulse,
    /// Pin 上输出的是RTC clock 信号
    Clock,
}

#[derive(PartialEq)]
pub enum RtcClock {
    LSI,
    LSE,
    HSE_DIV_32,
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

#[derive(Debug)]
pub enum Error {
    Clock,
    Timeout,
}

pub struct AnyRtc<'d, T: Instance, M: Mode> {
    t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

impl<'d, T: Instance, M: Mode> AnyRtc<'d, T, M> {
    pub fn new(_rtc: impl Peripheral<P = T>, config: Config) -> Result<Self, Error> {
        // 开启时钟
        T::id().open();
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

        // 等待可配置
        wait_for_true_timeout_block(100000, || T::configurable()).map_err(|_| Error::Timeout)?;

        // 等待rtc 寄存器同步
        wait_for_true_timeout_block(100000, || T::is_registers_synchronized())
            .map_err(|_| Error::Timeout)?;

        T::set_configurable(true);

        // 重新写入计数值
        if let Some(load) = config.load {
            T::set_counter(load);
        }

        T::set_configurable(false);
        pwr::rtc_unlock(false);

        Ok(())
    }

    #[inline]
    pub fn read(&self) -> u32 {
        T::get_counter()
    }

    #[inline]
    pub fn enable_interrupt(&self, en: bool) {
        unsafe {
            if en {
                cortex_m::peripheral::NVIC::unmask(interrupt::RTC)
            } else {
                cortex_m::peripheral::NVIC::mask(interrupt::RTC)
            }
        }
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
    pub async fn wait_second(&self, s: u32) {
        let br = self.read() + s - 1;
        WakeFuture::<T>::new(EventKind::Alarm(br)).await
    }
}

#[derive(PartialEq)]
pub enum EventKind {
    Alarm(u32),
    Second,
}

static WAKER: [AtomicWaker; 1] = [AtomicWaker::new()];

pub struct WakeFuture<T: Instance> {
    _t: PhantomData<T>,
    event: EventKind,
}

impl<T: Instance> WakeFuture<T> {
    pub fn new(event: EventKind) -> Self {
        pwr::rtc_unlock(true);

        wait_for_true_timeout_block(1000, || T::configurable()).unwrap();
        // 等待rtc 寄存器同步
        wait_for_true_timeout_block(100000, || T::is_registers_synchronized())
            .map_err(|_| Error::Timeout)
            .unwrap();

        T::set_configurable(true);
        match event {
            EventKind::Alarm(v) => {
                T::clear_alarm();
                T::set_alarm(v);
                T::enable_alarm_interrupt(true);
            }
            EventKind::Second => T::enable_second_interrupt(true),
        }
        T::set_configurable(false);
        pwr::rtc_unlock(false);
        Self {
            _t: PhantomData,
            event,
        }
    }

    fn on_interrupt() {
        pwr::rtc_unlock(true);
        T::enable_alarm_interrupt(false);
        pwr::rtc_unlock(false);

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
        match self.event {
            EventKind::Alarm(_v) => {
                if T::is_alarm() {
                    T::clear_alarm();
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
            EventKind::Second => {
                if T::second_flag() {
                    T::clear_second_flag();
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

impl<T: Instance> Drop for WakeFuture<T> {
    fn drop(&mut self) {
        defmt::debug!("derop");
        pwr::rtc_unlock(true);
        match self.event {
            EventKind::Alarm(_) => T::enable_alarm_interrupt(false),
            EventKind::Second => T::enable_second_interrupt(false),
        }
        T::set_configurable(false);
        pwr::rtc_unlock(false);
    }
}

#[interrupt]
fn RTC() {
    critical_section::with(|_cs| WakeFuture::<RTC>::on_interrupt())
}
