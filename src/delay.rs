use cortex_m::asm::delay;

use crate::clock::sys_core_clock;

pub fn delay_us(us: usize) {
    let sys_clock = sys_core_clock();
    let cnt = if sys_clock > 24_000_000 {
        8
    } else if sys_clock > 18_000_000 {
        6
    } else if sys_clock > 8_000_000 {
        4
    } else {
        1
    };

    for _ in 0..us {
        // 16M： 4
        delay(cnt);
    }
}

pub fn delay_ms(ms: usize) {
    for _ in 0..ms {
        delay_us(1000);
    }
}

pub fn delay_s(s: usize) {
    for _ in 0..s {
        delay_us(1_000_000);
    }
}

#[derive(Debug)]
pub enum Error {
    Timeout,
}

#[inline]
pub fn wait_for_true_timeout_block<F>(timeout_tick: usize, f: F) -> Result<(), Error>
where
    F: Fn() -> bool,
{
    for _ in 0..timeout_tick {
        if f() {
            return Ok(());
        }
        cortex_m::asm::delay(1);
    }
    Err(Error::Timeout)
}

// pub struct Delay;

// impl Delay {
//     pub fn delay_ms(&self, ms: usize) {
//         delay(4000 * ms as u32)
//     }
// }

// impl embedded_hal::blocking::delay::DelayMs<usize> for Delay {
//     fn delay_ms(&mut self, _ms: usize) {
//         delay(400)
//     }
// }
