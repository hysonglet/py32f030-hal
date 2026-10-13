use crate::clock;
use embedded_hal::spi::{Mode, Phase, Polarity};

/// SPI error
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Error {
    /// Initialization failed
    Init,
    /// Read failed
    Read,
    /// Write failed
    Write,
    /// Wait timeout
    Timeout,
    /// Bus busy
    Busy,
    /// Overrun (OVR)
    Overrun,
    /// Mode fault (MODF, NSS pulled low)
    ModeFault,
    /// DMA transfer error (or DMA interface used without a configured DMA channel)
    Dma,
}

/// Bidirectional data mode
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum BidirectionalMode {
    /// Two-line unidirectional
    #[default]
    Line2Unidirectional = 0,
    /// One-line bidirectional
    Line1Bidirectional = 1,
}

/// Frame format (MSB / LSB first)
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum BitOrder {
    /// Most significant bit first
    #[default]
    MSB = 0,
    /// Least significant bit first
    LSB = 1,
}

/// Baud rate divider
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum BaudRateDiv {
    /// fPCLK/2
    #[default]
    Div2 = 0,
    /// fPCLK/4
    Div4 = 1,
    /// fPCLK/8
    Div8 = 2,
    /// fPCLK/16
    Div16 = 3,
    /// fPCLK/32
    Div32 = 4,
    /// fPCLK/64
    Div64 = 5,
    /// fPCLK/128
    Div128 = 6,
    /// fPCLK/256
    Div256 = 7,
}

impl BaudRateDiv {
    /// Return the actual baud rate.
    pub fn baud_rate(&self) -> u32 {
        clock::sys_pclk() / (0x02 << (*self as usize))
    }
}

impl From<u8> for BaudRateDiv {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Div2,
            1 => Self::Div4,
            2 => Self::Div8,
            3 => Self::Div16,
            4 => Self::Div32,
            5 => Self::Div64,
            6 => Self::Div128,
            7 => Self::Div256,
            _ => unreachable!("value({}) must <= 7", value),
        }
    }
}

/// Master/slave role
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum Rule {
    /// Slave
    Slave = 0,
    /// Master
    #[default]
    Master = 1,
}

/// Slave speed mode
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum SlaveSpeedMode {
    /// Normal mode (up to pclk/4)
    #[default]
    Normal = 0,
    /// Fast mode
    Fast = 1,
}

/// Data frame length
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum DataLength {
    /// 8 bits
    #[default]
    Length8 = 0,
    /// 16 bits
    Length16 = 1,
}

/// SPI configuration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// Clock polarity and phase (CPOL/CPHA)
    pub mode: Mode,
    /// Frame format (MSB / LSB first)
    pub bit_order: BitOrder,
    /// Baud rate divider
    pub baud_rate_div: BaudRateDiv,
    /// Data frame length (8 / 16 bits)
    pub data_len: DataLength,
    /// Bidirectional data mode; in one-line mode MOSI is used for both TX and RX
    pub bidirectional: BidirectionalMode,
    /// Receive only (output disabled, RXONLY)
    pub rx_only: bool,
    /// Slave speed mode
    pub slave_speed_mode: SlaveSpeedMode,
    /// Enable SS output (SSOE); in master mode NSS is driven by hardware
    pub ss_output_enable: bool,
    /// Software slave management (SSM). Must be true when NSS is not connected,
    /// otherwise a spurious MODF is reported
    pub software_slave_management: bool,
    /// RX FIFO threshold: true raises RXNE at 8 bits, false at 16 bits
    pub fifo_rx_threshold_8bit: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            bit_order: BitOrder::MSB,
            baud_rate_div: BaudRateDiv::Div2,
            data_len: DataLength::Length8,
            bidirectional: BidirectionalMode::Line2Unidirectional,
            rx_only: false,
            slave_speed_mode: SlaveSpeedMode::Normal,
            ss_output_enable: false,
            software_slave_management: false,
            fifo_rx_threshold_8bit: true,
        }
    }
}

impl Config {
    // ---- builder ----
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }
    pub fn bit_order(mut self, bit_order: BitOrder) -> Self {
        self.bit_order = bit_order;
        self
    }
    pub fn baud_rate_div(mut self, baud_rate_div: BaudRateDiv) -> Self {
        self.baud_rate_div = baud_rate_div;
        self
    }
    pub fn data_len(mut self, data_len: DataLength) -> Self {
        self.data_len = data_len;
        self
    }
    pub fn bidirectional(mut self, bidirectional: BidirectionalMode) -> Self {
        self.bidirectional = bidirectional;
        self
    }
    pub fn rx_only(mut self, rx_only: bool) -> Self {
        self.rx_only = rx_only;
        self
    }
    pub fn slave_speed_mode(mut self, slave_speed_mode: SlaveSpeedMode) -> Self {
        self.slave_speed_mode = slave_speed_mode;
        self
    }
    pub fn ss_output_enable(mut self, en: bool) -> Self {
        self.ss_output_enable = en;
        self
    }
    pub fn software_slave_management(mut self, en: bool) -> Self {
        self.software_slave_management = en;
        self
    }
    pub fn fifo_rx_threshold_8bit(mut self, en: bool) -> Self {
        self.fifo_rx_threshold_8bit = en;
        self
    }
}
