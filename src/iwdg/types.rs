//! Independent watchdog (IWDG) public types and configuration

/// Maximum reload value
pub const RELOAD_MAX: u16 = 0xfff;
/// Internal watchdog clock frequency (Hz)
pub const IWDG_CLOCK_HZ: u32 = 32768;

/// Prescaler divisor
#[derive(Clone, Copy)]
pub enum Div {
    Div4 = 0,
    Div8 = 1,
    Div16 = 2,
    Div32 = 3,
    Div64 = 4,
    Div128 = 5,
    Div256 = 6,
}

/// Watchdog configuration
#[derive(Clone, Copy)]
pub struct Config {
    /// Prescaler divisor
    pub div: Div,
    /// Reload value
    pub reload: u16,
    /// Window value (0 disables windowed mode).
    ///
    /// When enabled, the reload (feed) must occur while the counter is below the
    /// window value, otherwise a reset is triggered. Must satisfy
    /// `reload < window < 0xFFF`.
    pub window: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            div: Div::Div32,
            reload: 2048,
            window: 0,
        }
    }
}

impl Config {
    /// Compute the timeout in microseconds.
    pub fn timeout_us(&self) -> u32 {
        let hz = IWDG_CLOCK_HZ >> (self.div as usize + 2);
        1_000_000 * self.reload as u32 / hz
    }
}
