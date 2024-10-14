#[derive(Debug)]
pub enum Error {
    LsiTimeout,
    HseTimeout,
    PllTimeout,
    SysTimeout,
}

/// HSI 频率选择
#[derive(PartialEq)]
pub enum HsiHz {
    /// 000: 4MHz
    MHz4 = 0x00,
    /// 001: 8MHz
    MHz8 = 0x01,
    /// 010: 16MHz
    MHz16 = 0x03,
    /// 011: 22.12Mhz
    MHz22_12 = 0x04,
    /// 100: 24MHz
    MHz24 = 0x05,
}

impl From<u8> for HsiHz {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::MHz4,
            1 => Self::MHz8,
            2 => Self::MHz16,
            3 => Self::MHz22_12,
            4 => Self::MHz16,
            5 => Self::MHz22_12,
            6 => Self::MHz24,
            _ => Self::MHz4,
        }
    }
}

pub(crate) enum HsiDiv {
    DIV1 = 0,
    DIV2 = 1,
    DIV4 = 2,
    DIV8 = 3,
    DIV16 = 4,
    DIV32 = 5,
    DIV64 = 6,
    DIV128 = 7,
}

impl From<u32> for HsiDiv {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::DIV1,
            2 => Self::DIV2,
            4 => Self::DIV4,
            8 => Self::DIV8,
            16 => Self::DIV16,
            32 => Self::DIV32,
            64 => Self::DIV64,
            128 => Self::DIV128,
            _ => unreachable!(), // _ => panic!("HSI DIV only allowd in [1, 2, 4, 8, 32, 64, 128]"),
        }
    }
}

/// PLK 时钟分频
#[derive(Clone, Copy, PartialEq)]
pub enum PclkDiv {
    Div1 = 1,
    Div2 = 4,
    Div4 = 5,
    Div8 = 6,
    Div16 = 7,
}

/// Hcl 时钟分频
#[derive(Clone, Copy)]
pub enum HclkDiv {
    Div1 = 1,
    Div2 = 8,
    Div4 = 9,
    Div8 = 10,
    Div16 = 11,
    Div64 = 12,
    Div128 = 13,
    Div256 = 14,
    Div512 = 15,
}

impl From<u8> for PclkDiv {
    fn from(value: u8) -> Self {
        match value {
            0..=3 => Self::Div1,
            4 => Self::Div2,
            5 => Self::Div4,
            6 => Self::Div8,
            7 => Self::Div16,
            _ => panic!(),
        }
    }
}

impl From<u8> for HclkDiv {
    fn from(value: u8) -> Self {
        match value {
            0..=7 => Self::Div1,
            8 => Self::Div2,
            9 => Self::Div4,
            10 => Self::Div8,
            11 => Self::Div16,
            12 => Self::Div64,
            13 => Self::Div128,
            14 => Self::Div256,
            15 => Self::Div512,
            _ => panic!(),
        }
    }
}

impl HclkDiv {
    pub(crate) fn div(&self) -> u32 {
        match *self {
            Self::Div1 => 1,
            Self::Div2 => 2,
            Self::Div4 => 4,
            Self::Div8 => 8,
            Self::Div16 => 16,
            Self::Div64 => 64,
            Self::Div128 => 128,
            Self::Div256 => 256,
            Self::Div512 => 512,
        }
    }
}

/// MCO 输出选择器
pub enum McoSelect {
    Disable = 0,
    SysClk = 1,
    Hsi = 3,
    Hse = 4,
    Pll = 5,
    Lsi = 6,
    Lse = 7,
}
