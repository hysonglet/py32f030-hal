use enumset::EnumSetType;

/// LPTIM error (current operations cannot fail).
pub enum Error {}

/// Prescaler (LPTIM_CFGR.PRESC[2:0]).
#[derive(PartialEq, Clone, Copy)]
pub enum Prescaler {
    /// Divide by 1
    Div1 = 0,
    /// Divide by 2
    Div2 = 1,
    /// Divide by 4
    Div4 = 2,
    /// Divide by 8
    Div8 = 3,
    /// Divide by 16
    Div16 = 4,
    /// Divide by 32
    Div32 = 5,
    /// Divide by 64
    Div64 = 6,
    /// Divide by 128
    Div128 = 7,
}

impl Prescaler {
    /// Returns the prescaler divisor.
    pub fn divisor(&self) -> u32 {
        1u32 << (*self as u8)
    }
}

/// LPTIM core clock source (RCC_CCIPR.LPTIMSEL[1:0]).
#[derive(PartialEq, Clone, Copy)]
pub enum ClockSource {
    /// APB clock (PCLK). Note: when PCLK is selected, the prescaler must be >= 2.
    Pclk = 0b00,
    /// Internal low-speed clock LSI (~32 kHz)
    Lsi = 0b01,
    /// External low-speed clock LSE (~32.768 kHz)
    Lse = 0b11,
}

/// LPTIM configuration.
///
/// Note: when `Lsi`/`Lse` is selected, the corresponding oscillator must first be
/// enabled by the user via `crate::clock::LSI` / `crate::clock::LSE`, otherwise LPTIM
/// has no core clock and will not count.
#[derive(PartialEq, Clone, Copy)]
pub struct LpTimerConfig {
    /// Core clock source.
    pub clock_source: ClockSource,
    /// Prescaler.
    pub prescaler: Prescaler,
    /// ARR preload enable (PRELOAD).
    pub preload: bool,
}

impl Default for LpTimerConfig {
    fn default() -> Self {
        Self {
            clock_source: ClockSource::Lsi,
            prescaler: Prescaler::Div1,
            preload: false,
        }
    }
}

/// LPTIM events.
#[derive(EnumSetType, Debug)]
pub enum Event {
    /// Auto-reload match (ARRM)
    AutoreloadMatch,
}
