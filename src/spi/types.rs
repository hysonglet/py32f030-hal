use crate::clock;

#[derive(PartialEq, Debug)]
pub enum Error {
    Init,
    Read,
    Write,
    Timeout,
    Busy,
}

/// Bidirectional data mode enable
#[derive(PartialEq)]
pub enum BidirectionalMode {
    /// 0: 2-line unidirectional data mode
    Line2Unidirectional = 0,
    /// 1: 1-line bidirectional data mode
    Line1Bidirectional = 1,
}

/// Frame format
///
/// This bit should not be changed when communication is ongoing
#[derive(PartialEq)]
pub enum BitOrder {
    /// 0: Data is transmitted with the MSB first.
    MSB = 0,
    /// 1: Data is transmitted with the LSB first.
    LSB = 1,
}

/// Baud rate control
///
/// These bits should not be changed when communication is ongoing.
/// Note: In slave mode, the fastest baud rate only supports fPCLK/4
#[derive(PartialEq, Clone, Copy)]
pub enum BaudRateDiv {
    /// 000: fPCLK/2
    Div2 = 0,
    /// 001: fPCLK/4
    Div4 = 1,
    /// 010: fPCLK/8
    Div8 = 2,
    /// 011: fPCLK/16
    Div16 = 3,
    /// 100: fPCLK/32
    Div32 = 4,
    /// 101: fPCLK/64
    Div64 = 5,
    /// 110: fPCLK/128
    Div128 = 6,
    /// 111: fPCLK/256
    Div256 = 7,
}

impl BaudRateDiv {
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

#[derive(PartialEq, Clone, Copy)]
pub enum Rule {
    /// 0: Slave configuration
    Slave = 0,
    /// 1: Master configuration
    Master = 1,
}

/// Slave fast mode enable
#[derive(PartialEq)]
pub enum SlaveSpeedMode {
    /// 0: Slave normal mode, the speed of the slave mode supporting the fastest SPI clock is less than pclk/4
    Normal = 0,
    /// 1: Slave fast mode, can support SPI clock speed in slave mode up to pclk/4
    Fast = 1,
}

/// SPI transmission data length
#[derive(PartialEq)]
pub enum DataLength {
    /// 0: 8-bit data frame transmission
    Length8 = 0,
    /// 1: 16-bit data frame transmission
    Length16 = 1,
}
