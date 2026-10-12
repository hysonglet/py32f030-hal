use enumset::EnumSetType;

/// 定时器错误（当前操作无失败分支）
#[derive(Debug)]
pub enum Error {}

/// 统一的 APB 定时器标识（用于时钟索引、中断向量、异步 Waker 数组索引）
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerId {
    TIM1 = 0,
    TIM3 = 1,
    TIM14 = 2,
    TIM16 = 3,
    /// TIM17 与 TIM16 功能完全一致，共用同一套寄存器布局
    TIM17 = 4,
}

/// 输入捕获和输出 pwm 通道（统一枚举；单通道定时器仅 CH1 有效）
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Channel {
    CH1,
    CH2,
    CH3,
    CH4,
}

impl Channel {
    /// 通道序号（0 起）
    #[inline]
    pub fn index(self) -> usize {
        match self {
            Channel::CH1 => 0,
            Channel::CH2 => 1,
            Channel::CH3 => 2,
            Channel::CH4 => 3,
        }
    }

    /// 由序号构造通道，越界返回 None
    #[inline]
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Channel::CH1),
            1 => Some(Channel::CH2),
            2 => Some(Channel::CH3),
            3 => Some(Channel::CH4),
            _ => None,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ChannelOutput {
    /// 主输出
    P,
    /// 互补输出（仅 TIM1 / TIM16 支持）
    N,
}

/// 通道类型
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ChannelType {
    /// 通道被配置为输出
    Pwm = 0,
    /// 通道被配置为输入，ICx 映射在 TIx 上
    Capture1 = 1,
    /// 通道被配置为输入，ICx 映射在 TIy 上
    Capture2 = 2,
    /// 通道被配置为输入，ICx 映射在 TRC 上
    Capture3 = 3,
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum ChannelMode {
    /// 冻结：CCRx 与 CNT 的比较对 OCxREF 不起作用
    #[default]
    Mode0 = 0,
    /// 匹配时强制 OCxREF 为高
    Mode1 = 1,
    /// 匹配时强制 OCxREF 为低
    Mode2 = 2,
    /// 匹配时翻转 OCxREF
    Mode3 = 3,
    /// 强制为无效电平
    Mode4 = 4,
    /// 强制为有效电平
    Mode5 = 5,
    /// PWM 模式 1
    PWM1 = 6,
    /// PWM 模式 2
    PWM2 = 7,
}

/// 记数方向
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum CountDirection {
    /// 向上计数
    Up = 0,
    /// 向下计数
    Down = 1,
}

/// 中央对齐模式
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CenterAlignedMode {
    /// 边沿对齐
    EdgeAligned = 0,
    /// 中央对齐模式 1
    CenterAligned1 = 1,
    /// 中央对齐模式 2
    CenterAligned2 = 2,
    /// 中央对齐模式 3
    CenterAligned3 = 3,
}

/// 软件触发事件（EGR）
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Triggle {
    UG = 0,
    CC1G = 1,
    CC2G = 2,
    CC3G = 3,
    CC4G = 4,
    COMG = 5,
    TG = 6,
    BG = 7,
}

/// 定时器事件（统一枚举；不支持的事件在对应定时器上恒为 false / 配置无效）
#[derive(EnumSetType, Debug)]
pub enum Event {
    /// 更新中断标记
    UIF,
    /// 捕获/比较 1 中断标记
    CC1IF,
    /// 捕获/比较 2 中断标记
    CC2IF,
    /// 捕获/比较 3 中断标记
    CC3IF,
    /// 捕获/比较 4 中断标记
    CC4IF,
    /// COM 中断标记
    COMIF,
    /// 触发中断标记
    TIF,
    /// 刹车中断标记
    BIF,
    /// 捕获/比较 1 过捕获标记
    CC1OF,
    /// 捕获/比较 2 过捕获标记
    CC2OF,
    /// 捕获/比较 3 过捕获标记
    CC3OF,
    /// 捕获/比较 4 过捕获标记
    CC4OF,
}

/// 时钟分频因子（tDTS）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClockDiv {
    /// tDTS = tCK_INT
    Div1 = 0,
    /// tDTS = 2 x tCK_INT
    Div2 = 1,
    /// tDTS = 4 x tCK_INT
    Div4 = 2,
}

/// 输入捕获极性
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputCapturePolarity {
    /// 上升沿捕获
    Rising = 0,
    /// 下降沿捕获
    Falling = 1,
    /// 双沿捕获
    BothEdge = 2,
}

/// 输入捕获选择（CCxS）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputCaptureSelection {
    /// 直连：ICx 映射到 TIx
    DirectTI = 1,
    /// 非直连：ICx 映射到 TIy
    IndirectTI = 2,
    /// ICx 映射到 TRC
    TRC = 3,
}

/// 输入捕获预分频
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputCapturePrescaler {
    Div1 = 0,
    Div2 = 1,
    Div4 = 2,
    Div8 = 3,
}

/// 输入捕获数字滤波器
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InputCaptureFilter(pub u8);

impl InputCaptureFilter {
    pub fn none() -> Self {
        Self(0)
    }
    pub fn value(&self) -> u8 {
        self.0 & 0x0f
    }
}

/// 编码器模式
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EncoderMode {
    /// 仅在 TI1 边沿计数（x2）
    Ti1 = 1,
    /// 仅在 TI2 边沿计数（x2）
    Ti2 = 2,
    /// TI1 与 TI2 边沿都计数（x4）
    Ti12 = 3,
}

/// 从模式选择（SMS）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlaveMode {
    /// 禁止从模式
    Disable = 0,
    /// 编码器模式 1
    EncoderTi1 = 1,
    /// 编码器模式 2
    EncoderTi2 = 2,
    /// 编码器模式 3
    EncoderTi12 = 3,
    /// 复位模式
    Reset = 4,
    /// 门控模式
    Gated = 5,
    /// 触发模式
    Trigger = 6,
    /// 外部时钟模式 1（仅 TIM1 有 ETR）
    ExternalClock1 = 7,
}

/// 触发输入选择（TS）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TriggerSelection {
    ITR0 = 0,
    ITR1 = 1,
    ITR2 = 2,
    ITR3 = 3,
    /// TI1 边沿检测器
    TI1F_ED = 4,
    /// 滤波后的 TI1FP1
    TI1FP1 = 5,
    /// 滤波后的 TI2FP2
    TI2FP2 = 6,
    /// 外部触发（仅 TIM1 支持）
    ETRF = 7,
}

/// 主模式触发输出（TRGO，MMS）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MasterOutputTrigger {
    Reset = 0,
    Enable = 1,
    Update = 2,
    Oc1 = 3,
    Oc1Ref = 4,
    Oc2Ref = 5,
    Oc3Ref = 6,
    Oc4Ref = 7,
}

/// OCREF 清除源
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OcRefClearSource {
    /// 使用 ETR 作为 OCREF_CLR 输入
    Etc = 1,
    /// 不使用
    None = 0,
}

/// 死区时间（DTG）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DeadTime(pub u8);

impl DeadTime {
    pub fn value(&self) -> u8 {
        self.0
    }
}

/// 锁定级别
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LockLevel {
    Off = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
}

/// 刹车/空闲时输出关闭行为
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OffState {
    /// 输出禁止
    Disable = 0,
    /// 输出使能，由空闲态决定
    Idle = 1,
}

/// 刹车极性
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BreakPolarity {
    /// 低电平有效
    Low = 0,
    /// 高电平有效
    High = 1,
}

/// TIM14 TI1 输入重映射（TIM14_OR）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ti1Remap {
    /// TI1 来自 TIM14_CH1 引脚
    Channel1 = 0,
    /// TI1 来自备选输入
    Alternative = 1,
}

/// 单脉冲模式
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OnePulseMode {
    /// 单次：发生更新事件后计数器停止（OPM=1）
    Single = 1,
    /// 重复：计数器持续运行（OPM=0）
    Repetitive = 0,
}

////////////////////// 配置结构 //////////////////////

/// 通道输出配置（极性 / 使能 / 空闲态）
#[derive(Default, Clone, Copy)]
pub struct ChannelOutputConfig {
    /// 输出使能
    pub state: bool,
    /// 有效极性：true 为高电平有效
    pub polarity: bool,
    /// 空闲态（OIS，仅 TIM1 / TIM16 有效）
    pub idle_state: bool,
}

/// 通道配置（输出比较 / PWM）
#[derive(Default, Clone, Copy)]
pub struct ChannelConfig {
    /// 通道模式
    pub mode: ChannelMode,
    /// 是否允许外部事件清除 OCREF
    pub clear: bool,
    /// 输出比较快速使能
    pub fast: bool,
    /// 输出比较预装载使能
    pub preload: bool,
    /// 捕获/比较值（占空比）
    pub compare: u16,
    /// 主输出（CHx）配置
    pub ch: Option<ChannelOutputConfig>,
    /// 互补输出（CHxN）配置，仅 TIM1 / TIM16 有效
    pub n_ch: Option<ChannelOutputConfig>,
}

impl ChannelConfig {
    pub fn mode(mut self, mode: ChannelMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn compare(mut self, compare: u16) -> Self {
        self.compare = compare;
        self
    }
    pub fn clear(mut self, clear: bool) -> Self {
        self.clear = clear;
        self
    }
    pub fn fast(mut self, fast: bool) -> Self {
        self.fast = fast;
        self
    }
    pub fn preload(mut self, preload: bool) -> Self {
        self.preload = preload;
        self
    }
    /// 设置主输出配置
    pub fn ch(mut self, ch: ChannelOutputConfig) -> Self {
        self.ch = Some(ch);
        self
    }
    /// 设置互补输出配置（仅 TIM1 / TIM16）
    pub fn n_ch(mut self, n_ch: ChannelOutputConfig) -> Self {
        self.n_ch = Some(n_ch);
        self
    }
}

/// 制动与死区配置（仅 TIM1 / TIM16）
#[derive(Clone, Copy)]
pub struct BreakConfig {
    pub dead_time: DeadTime,
    pub lock_level: LockLevel,
    pub off_state_run: OffState,
    pub off_state_idle: OffState,
    pub break_polarity: BreakPolarity,
    pub break_enable: bool,
    pub automatic_output: bool,
}

impl Default for BreakConfig {
    fn default() -> Self {
        Self {
            dead_time: DeadTime(0),
            lock_level: LockLevel::Off,
            off_state_run: OffState::Disable,
            off_state_idle: OffState::Disable,
            break_polarity: BreakPolarity::Low,
            break_enable: false,
            automatic_output: true,
        }
    }
}
