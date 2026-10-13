//! Power management (PWR) public types and configuration

/// Core voltage scaling level (CR1.VOS)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VoltageScale {
    /// Level 1 (highest performance, highest power)
    Range1 = 0,
    /// Level 2 (low power)
    Range2 = 1,
    /// Level 3 (lowest power)
    Range3 = 2,
}

/// Programmable voltage detector (PVD) configuration
///
/// The threshold is selected by a 3-bit value (0–7), mapped to the 1.9V–3.3V
/// range; see the reference manual PVD threshold table.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PvdConfig {
    /// Whether to enable the PVD
    pub enable: bool,
    /// Threshold level (0–7)
    pub threshold: u8,
    /// Enable PVD rising/falling edge filtering
    pub filter: bool,
    /// Filtering time (CR2.FLT_TIME, 0–7)
    pub filter_time: u8,
    /// Trigger an interrupt when the PVD output falls below the threshold
    pub interrupt: bool,
}
