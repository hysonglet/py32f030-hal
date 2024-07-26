use atomic_polyfill::{AtomicU64, AtomicU8, Ordering};
use core::cell::Cell;

use crate::clock::sys_core_clock;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::exception;
use critical_section::{CriticalSection, Mutex};
use embassy_time_driver::{AlarmHandle, Driver, TICK_HZ};

pub const ALARM_COUNT: usize = 1;

type AlarmStateCallBackType = (fn(*mut ()), *mut ());

struct AlarmState {
    timestamp: Cell<u64>,
    callback: Cell<Option<AlarmStateCallBackType>>,
}

unsafe impl Send for AlarmState {}

impl AlarmState {
    const fn new() -> Self {
        Self {
            timestamp: Cell::new(u64::MAX),
            callback: Cell::new(None),
        }
    }
}
/// 系统时钟
pub struct SystickDriver {
    alarm_count: AtomicU8,
    alarms: Mutex<[AlarmState; ALARM_COUNT]>,
    period_count: AtomicU64,
}

// const ALARM_STATE_NEW: AlarmState = AlarmState::new();

embassy_time_driver::time_driver_impl!(static DRIVER: SystickDriver = SystickDriver {
    alarm_count: AtomicU8::new(0),
    alarms: Mutex::new([AlarmState::new(); ALARM_COUNT]),
    period_count: AtomicU64::new(0)
});

impl SystickDriver {
    // const SYST_COUNTER_MASK: u32 = 0x00ff_ffff;
    const SYST_CSR_ENABLE: u32 = 1 << 0;
    const SYST_CSR_TICKINT: u32 = 1 << 1;
    const SYST_CSR_CLKSOURCE: u32 = 1 << 2;
    // const SYST_CSR_COUNTFLAG: u32 = 1 << 16;
    // const SYST_CALIB_SKEW: u32 = 1 << 30;
    // const SYST_CALIB_NOREF: u32 = 1 << 31;

    fn block() -> &'static cortex_m::peripheral::syst::RegisterBlock {
        unsafe { cortex_m::peripheral::SYST::PTR.as_ref().unwrap() }
    }

    #[inline]
    pub fn set_clock_source(clk_source: SystClkSource) {
        match clk_source {
            SystClkSource::External => unsafe {
                Self::block().csr.modify(|v| v & !Self::SYST_CSR_CLKSOURCE)
            },
            SystClkSource::Core => unsafe {
                Self::block().csr.modify(|v| v | Self::SYST_CSR_CLKSOURCE)
            },
        }
    }

    fn enable_counter() {
        unsafe { Self::block().csr.modify(|v| v | Self::SYST_CSR_ENABLE) }
    }

    // fn disable_counter() {
    //     unsafe { Self::block().csr.modify(|v| v & !Self::SYST_CSR_ENABLE) }
    // }

    /// Enables SysTick interrupt
    #[inline]
    fn enable_interrupt() {
        unsafe { Self::block().csr.modify(|v| v | Self::SYST_CSR_TICKINT) }
    }

    #[inline]
    fn disable_interrupt() {
        unsafe { Self::block().csr.modify(|v| v & !Self::SYST_CSR_TICKINT) }
    }

    #[inline]
    fn clear_current() {
        unsafe { Self::block().cvr.write(0) }
    }

    #[inline]
    fn set_reload(value: u32) {
        unsafe { Self::block().rvr.write(value) }
    }
}

impl SystickDriver {
    fn init(&'static self) {
        Self::disable_interrupt();
        Self::set_clock_source(SystClkSource::Core);
        Self::clear_current();

        let cnt_per_tick = sys_core_clock() / TICK_HZ as u32;

        Self::set_reload(cnt_per_tick);

        critical_section::with(|_| {
            Self::enable_interrupt();
            Self::enable_counter();
        })
    }
    fn trigger_alarm(&self, n: usize, cs: CriticalSection) {
        let alarm = &self.alarms.borrow(cs)[n];
        alarm.timestamp.set(u64::MAX);

        if let Some((f, ctx)) = alarm.callback.get() {
            f(ctx)
        }
    }

    #[inline(always)]
    fn on_interrupt(&self, n: usize) {
        // Self::disable_counter();
        self.period_count.fetch_add(1, Ordering::Relaxed);
        critical_section::with(|cs| {
            let alarm = &self.alarms.borrow(cs)[n];
            let timestamp = alarm.timestamp.get();
            // 闹钟触发了
            if timestamp <= self.now() {
                self.trigger_alarm(n, cs)
            }
        });
        // Self::enable_interrupt();
    }
}

impl Driver for SystickDriver {
    fn now(&self) -> u64 {
        self.period_count.load(Ordering::Relaxed)
    }

    unsafe fn allocate_alarm(&self) -> Option<AlarmHandle> {
        let id = self
            .alarm_count
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |x| {
                if x < ALARM_COUNT as u8 {
                    Some(x + 1)
                } else {
                    None
                }
            });

        id.map_or_else(|_| None, |id| Some(AlarmHandle::new(id)))
    }

    fn set_alarm_callback(&self, alarm: AlarmHandle, callback: fn(*mut ()), ctx: *mut ()) {
        let n = alarm.id() as usize;
        critical_section::with(|cs| {
            let alarm = &self.alarms.borrow(cs)[n];
            alarm.callback.set(Some((callback, ctx)));
        })
    }

    fn set_alarm(&self, alarm: AlarmHandle, timestamp: u64) -> bool {
        let n = alarm.id() as usize;

        critical_section::with(|cs| {
            let alarm = &self.alarms.borrow(cs)[n];
            alarm.timestamp.set(timestamp);

            if timestamp <= self.now() {
                alarm.timestamp.set(u64::MAX);
                false
            } else {
                // 闹钟设置成功
                true
            }
        })
    }
}

pub(crate) fn init(_cs: CriticalSection) {
    DRIVER.init();
}

#[exception]
fn SysTick() {
    DRIVER.on_interrupt(0);
}
