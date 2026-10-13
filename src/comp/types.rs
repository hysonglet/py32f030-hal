/// Comparator hysteresis (CSR.HYST)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Hysteresis {
    /// No hysteresis
    #[default]
    None = 0,
    /// Low hysteresis
    Low = 1,
    /// Medium hysteresis
    Medium = 2,
    /// High hysteresis
    High = 3,
}

/// Comparator power/speed mode (CSR.PWRMODE)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PowerMode {
    /// High speed (highest power)
    #[default]
    HighSpeed = 0,
    /// Medium speed
    MediumSpeed = 1,
    /// Low speed (low power)
    LowSpeed = 2,
    /// Ultra-low speed (lowest power)
    UltraLowSpeed = 3,
}

/// Output polarity (CSR.POLARITY)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Polarity {
    /// Non-inverted output
    #[default]
    NonInverted = 0,
    /// Inverted output
    Inverted = 1,
}

/// Comparator configuration
///
/// `inpsel` / `inmsel` select the non-inverting / inverting input source
/// respectively. See the reference manual COMP input mapping table (available
/// pins differ by package); the raw register value is kept here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// Hysteresis
    pub hysteresis: Hysteresis,
    /// Power/speed mode
    pub power_mode: PowerMode,
    /// Output polarity
    pub polarity: Polarity,
    /// Non-inverting input select (CSR.INPSEL)
    pub inpsel: u8,
    /// Inverting input select (CSR.INMSEL)
    pub inmsel: u8,
    /// Enable the scaler (CSR.SCALER_EN)
    pub scaler_en: bool,
    /// Window mode (CSR.WINMODE; only valid for some input combinations)
    pub window_mode: bool,
    /// Filter enable (FR.FLTEN)
    pub filter_enable: bool,
    /// Filter count (FR.FLTCNT)
    pub filter_count: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hysteresis: Hysteresis::None,
            power_mode: PowerMode::HighSpeed,
            polarity: Polarity::NonInverted,
            inpsel: 0,
            inmsel: 0,
            scaler_en: false,
            window_mode: false,
            filter_enable: false,
            filter_count: 0,
        }
    }
}

impl Config {
    pub fn hysteresis(mut self, hysteresis: Hysteresis) -> Self {
        self.hysteresis = hysteresis;
        self
    }
    pub fn power_mode(mut self, power_mode: PowerMode) -> Self {
        self.power_mode = power_mode;
        self
    }
    pub fn polarity(mut self, polarity: Polarity) -> Self {
        self.polarity = polarity;
        self
    }
    pub fn inpsel(mut self, inpsel: u8) -> Self {
        self.inpsel = inpsel;
        self
    }
    pub fn inmsel(mut self, inmsel: u8) -> Self {
        self.inmsel = inmsel;
        self
    }
    pub fn scaler_en(mut self, en: bool) -> Self {
        self.scaler_en = en;
        self
    }
    pub fn window_mode(mut self, en: bool) -> Self {
        self.window_mode = en;
        self
    }
    pub fn filter(mut self, enable: bool, count: u8) -> Self {
        self.filter_enable = enable;
        self.filter_count = count;
        self
    }
}

/// COMP error
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Error {
    /// This instance does not support window mode
    WindowModeUnsupported,
}
