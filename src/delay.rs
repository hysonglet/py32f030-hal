use cortex_m::asm::delay;

// use embassy_time;

pub fn delay_us(us: usize) {
    for _ in 0..us {
        delay(1000);
    }
}

pub fn delay_ms(ms: usize) {
    for _ in 0..ms {
        delay_us(1000);
    }
}

pub fn delay_s(s: usize) {
    for _ in 0..s {
        delay_us(1000_1000);
    }
}

#[inline]
pub fn wait_for_flag_timeout<F>(timeout_us: usize, f: F) -> Result<(), ()>
where
    F: Fn() -> bool,
{
    for _ in 0..timeout_us {
        if f() {
            return Ok(());
        }
        delay_us(1);
    }
    return Err(());
}
