use enumset::EnumSetType;

/// LPTIM 错误（当前操作无失败分支）
pub enum Error {}

/// 预分频系数（LPTIM_CFGR.PRESC[2:0]）
#[derive(PartialEq, Clone, Copy)]
pub enum Prescaler {
    /// 1 分频
    Div1 = 0,
    /// 2 分频
    Div2 = 1,
    /// 4 分频
    Div4 = 2,
    /// 8 分频
    Div8 = 3,
    /// 16 分频
    Div16 = 4,
    /// 32 分频
    Div32 = 5,
    /// 64 分频
    Div64 = 6,
    /// 128 分频
    Div128 = 7,
}

impl Prescaler {
    /// 返回分频系数
    pub fn divisor(&self) -> u32 {
        1u32 << (*self as u8)
    }
}

/// LPTIM 内核时钟源（RCC_CCIPR.LPTIMSEL[1:0]）
#[derive(PartialEq, Clone, Copy)]
pub enum ClockSource {
    /// APB 时钟（PCLK）。注意：选 PCLK 时预分频必须 >= 2 分频。
    Pclk = 0b00,
    /// 内部低速时钟 LSI（约 32 kHz）
    Lsi = 0b01,
    /// 外部低速时钟 LSE（约 32.768 kHz）
    Lse = 0b11,
}

/// LPTIM 配置
///
/// 注意：选择 `Lsi`/`Lse` 时，需先由用户通过 `crate::clock::LSI` / `crate::clock::LSE`
/// 使能对应振荡器，否则 LPTIM 无内核时钟不会计数。
#[derive(PartialEq, Clone, Copy)]
pub struct LpTimerConfig {
    /// 内核时钟源
    pub clock_source: ClockSource,
    /// 预分频系数
    pub prescaler: Prescaler,
    /// ARR 预装载使能（PRELOAD）
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

/// LPTIM 事件
#[derive(EnumSetType, Debug)]
pub enum Event {
    /// 自动重装载匹配（ARRM）
    AutoreloadMatch,
}
