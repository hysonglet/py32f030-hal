/// 比较器迟滞（CSR.HYST）
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Hysteresis {
    /// 无迟滞
    #[default]
    None = 0,
    /// 低迟滞
    Low = 1,
    /// 中迟滞
    Medium = 2,
    /// 高迟滞
    High = 3,
}

/// 比较器功耗/速度模式（CSR.PWRMODE）
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PowerMode {
    /// 高速（功耗最高）
    #[default]
    HighSpeed = 0,
    /// 中速
    MediumSpeed = 1,
    /// 低速（低功耗）
    LowSpeed = 2,
    /// 极低速（最低功耗）
    UltraLowSpeed = 3,
}

/// 输出极性（CSR.POLARITY）
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Polarity {
    /// 同相输出
    #[default]
    NonInverted = 0,
    /// 反相输出
    Inverted = 1,
}

/// 比较器配置
///
/// `inpsel` / `inmsel` 分别为同相/反相输入源选择，取值请参考数据手册的
/// COMP 输入映射表（不同封装可选的输入引脚不同），此处保留原始寄存器值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// 迟滞
    pub hysteresis: Hysteresis,
    /// 功耗/速度模式
    pub power_mode: PowerMode,
    /// 输出极性
    pub polarity: Polarity,
    /// 同相输入选择（CSR.INPSEL）
    pub inpsel: u8,
    /// 反相输入选择（CSR.INMSEL）
    pub inmsel: u8,
    /// 使能缩放器（CSR.SCALER_EN）
    pub scaler_en: bool,
    /// 窗口模式（CSR.WINMODE，仅部分输入组合有效）
    pub window_mode: bool,
    /// 滤波使能（FR.FLTEN）
    pub filter_enable: bool,
    /// 滤波计数（FR.FLTCNT）
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

/// COMP 错误
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Error {
    /// 该实例不支持窗口模式
    WindowModeUnsupported,
}
