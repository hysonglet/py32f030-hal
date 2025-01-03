use embedded_time::rate::{Baud, Extensions};

#[derive(Debug)]
pub enum Error {
    StartTimeout,
    ReadTimeout,
    WriteTimeout,
    DMA,
    /// 噪音/校验/帧错误
    Noise,
    Frame,
    Parity,
    Others,
}

/// Serial configuration structure.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    /// Serial baudrate.
    pub baud_rate: Baud,
    /// Number of stop bits.
    pub stop_bit: StopBits,
    /// Parity settings.
    pub parity: Parity,
    // pub hw_flow_ctrl: HwFlowCtrl,
    /// Number of data bits.
    pub data_bits: DataBits,
    /// Oversampling type.
    pub over_sampling: OverSampling,
    // pub mode: T,
}

impl Default for Config {
    /// Serial configuration defaults to 115200 Bd, 8-bit data, no parity check, 1 stop bit, oversampling by 16.
    #[inline]
    fn default() -> Self {
        Self {
            baud_rate: 115200.Bd(),
            stop_bit: StopBits::One,
            parity: Parity::None,
            data_bits: DataBits::Eight,
            over_sampling: OverSampling::Sixteen,
        }
    }
}

/// Number of stop bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum StopBits {
    /// 1 stop bit.
    #[default]
    One = 0,
    /// 2 stop bits.
    Two = 1,
}

/// Parity check type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Parity {
    /// No parity checks.
    #[default]
    None = 0,
    /// Even parity check bit.
    Even = 1,
    /// Odd parity check bit.
    Odd = 2,
}

// /// 串口流控
// #[derive(Default)]
// pub enum HwFlowCtrl {
//     #[default]
//     None,
//     Rts = 1,
//     Cts = 2,
//     RtsCts = 3,
// }

/// Clock oversampling settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum OverSampling {
    /// Oversampling by 16.
    #[default]
    Sixteen = 0,
    /// Oversampling by 8.
    Eight = 1,
}

impl OverSampling {
    pub(crate) fn div(&self) -> u32 {
        if *self == Self::Sixteen {
            16
        } else {
            8
        }
    }
}

impl From<OverSampling> for bool {
    fn from(value: OverSampling) -> Self {
        value == OverSampling::Eight
    }
}

/// Number of data bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum DataBits {
    /// 8-bit data word.
    #[default]
    Eight = 0,
    /// 9-bit data word.
    Nine = 1,
}

impl From<DataBits> for bool {
    #[inline]
    fn from(value: DataBits) -> Self {
        value == DataBits::Nine
    }
}
