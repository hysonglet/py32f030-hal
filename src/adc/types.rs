use enumset::EnumSetType;

#[derive(Debug)]
pub enum Error {
    Busy,
    Timeout,
    Calibrate,
}

/// ADC clock mode, software can set and clear this bit to define the clock source of the analog ADC
#[derive(Debug)]
pub enum ClockMode {
    PCLK = 0,
    PLCK_DIV2 = 1,
    PCLK_DIV4 = 2,
    PCLK_DIV8 = 3,
    PCLK_DIV16 = 4,
    PCLK_DIV32 = 5,
    PCLK_DIV64 = 6,
    HSI = 0b1000,
    HSI_DIV2 = 0b1001,
    HSI_DIV4 = 0b1010,
    HSI_DIV8 = 0b1011,
    HSI_DIV16 = 0b1100,
    HSI_DIV32 = 0b1101,
    HSI_DIV64 = 0b1110,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AdcChannel {
    /// PA0
    Channel0 = 0,
    /// PA1
    Channel1 = 1,
    /// PA2
    Channel2 = 2,
    /// PA3
    Channel3 = 3,
    /// PA4
    Channel4 = 4,
    /// PA5
    Channel5 = 5,
    /// PA6
    Channel6 = 6,
    /// PA7
    Channel7 = 7,
    /// PB0
    Channel8 = 8,
    /// PB1
    Channel9 = 9,

    /// inner temperature
    Channel11 = 11,
    /// inner ref voltage
    Channel12 = 12,
}

/// discontinuous mode and Single/Continuous Conversion Mode
#[derive(PartialEq)]
pub enum ConversionMode {
    /// 单次转换模式 (CONT=0, DISCEN=0)
    Single,
    /// 连续转换模式 (CONT=1)
    Continuous,
    /// 非连续转换模式 (DISCEN=1)
    Discontinuous,
}

/// External trigger enable and polarity selection
#[derive(PartialEq)]
pub enum TrigleSignal {
    Soft,
    Rising(ExitTrigleSource),
    Falling(ExitTrigleSource),
    RisingFalling(ExitTrigleSource),
}

/// External trigger selection
#[derive(PartialEq)]
pub enum ExitTrigleSource {
    TIM1_TRG0 = 0,
    TIM1_CC4 = 1,
    TIM3_TRGP = 3,
}

/// Data alignment
#[derive(PartialEq)]
pub enum Align {
    Right,
    Left,
}

/// Data resolution
/// Software sets this bit to select the conversion resolution
pub enum Resolution {
    Bit12 = 0,
    Bit10 = 1,
    Bit8 = 2,
    Bit6 = 3,
}

/// Scan sequence direction
/// Software can set and clear this bit to select the scan sequence direction
#[derive(PartialEq)]
pub enum ScanDir {
    Up,
    Down,
}

/// DMA Configuration
/// This bit can be set and cleared by software, selects between two DMA modes of operation and is valid when DMAEN = 1
// #[derive(PartialEq)]
// pub enum DmaMode {
//     Single,
//     Cycle,
// }

/// Sampling time selection
/// Software configurable bit selects the sampling time for all channels
pub enum SampleCycles {
    Cycle_3_5 = 0,
    Cycle_5_5 = 1,
    Cycle_7_5 = 2,
    Cycle_13_5 = 3,
    Cycle_28_5 = 4,
    Cycle_41_5 = 5,
    Cycle_71_5 = 6,
    Cycle_239_5 = 7,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CalibrationSampleTime {
    Cycle_1 = 3,
    Cycle_2 = 0,
    Cycle_4 = 1,
    Cycle_8 = 2,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CalibrationSelect {
    Offset = 0,
    OffsetLinearity = 1,
}

#[derive(EnumSetType, Debug)]
pub enum Event {
    /// 采样结束标志
    EOSMP,
    /// 转换结束标志
    EOC,
    /// 序列结束标志
    EOSEQ,
    /// ADC 过载
    OVR,
    /// 模拟看门狗
    AWD,
}
