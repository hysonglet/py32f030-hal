//! Segment LED controller (LED) public types and configuration

use crate::clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex};

/// LED instance index (only LED1)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Id {
    Led1 = 0,
}

impl PeripheralIdToClockIndex for Id {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::Led1 => PeripheralClockIndex::LED,
        }
    }
}

/// Brightness control (CR.EHS, 2 bits)
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum LightControl {
    /// Darkest
    #[default]
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    /// Brightest
    Level3 = 3,
}

/// LED configuration
#[derive(Clone, Copy, Debug)]
pub struct LedConfig {
    /// Prescaler (0–255): scan clock = LED clock / (pr + 1)
    pub prescaler: u8,
    /// Number of COM lines (1–4)
    pub com_count: u8,
    /// Display timing t1
    pub t1: u8,
    /// Display timing t2
    pub t2: u8,
    /// Brightness control
    pub light_control: LightControl,
    /// Whether to enable the scan-complete interrupt
    pub interrupt: bool,
}

impl Default for LedConfig {
    fn default() -> Self {
        Self {
            prescaler: 0,
            com_count: 4,
            t1: 0,
            t2: 0,
            light_control: LightControl::Level3,
            interrupt: false,
        }
    }
}
