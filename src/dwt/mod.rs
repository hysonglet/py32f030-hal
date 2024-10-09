use crate::clock::sys_core_clock;
use cortex_m::peripheral::{DCB, DWT};

pub struct Dwt<const HZ: u32 = 8000000>;

#[derive(Debug)]
pub enum Error {
    // RepeatInitialization,
}

impl Dwt {
    pub fn init(dcb: &mut DCB, dwt: &mut DWT) -> Result<(), Error> {
        dcb.enable_trace();
        dwt.enable_cycle_counter();

        Ok(())
    }

    pub fn now() -> Instant {
        Instant {
            now: DWT::cycle_count(),
        }
    }

    pub fn delay_ms(ms: u32) {
        let ms = ms as u64 * clk::get_cpu_hz() as u64 / 1000;
        let now = Self::now();
        loop {
            if now.elapsed() as u64 > ms {
                break;
            }
        }
    }
}

/// A measurement of a monotonically non-decreasing clock
#[derive(Clone, Copy, Debug)]
pub struct Instant {
    now: u32,
}

impl Instant {
    /// Ticks elapsed since the `Instant` was created
    pub fn elapsed(self) -> u32 {
        DWT::cycle_count().wrapping_sub(self.now)
    }

    pub fn ms(&self) -> u32 {
        self.now / clk::get_cpu_hz()
    }
}
