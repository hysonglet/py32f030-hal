use embedded_time::rate::{Baud, Extensions};

/// 串口错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// 起始位超时
    StartTimeout,
    /// 读超时
    ReadTimeout,
    /// 写超时
    WriteTimeout,
    /// DMA 传输错误（或未配置 DMA 通道却使用 DMA 接口）
    DMA,
    /// 检测到噪声
    Noise,
    /// 帧错误
    Frame,
    /// 校验错误
    Parity,
    /// 溢出错误
    Overrun,
    /// 其它错误
    Others,
}

/// 停止位
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum StopBits {
    /// 1 个停止位
    #[default]
    One = 0,
    /// 2 个停止位
    Two = 1,
}

/// 校验方式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Parity {
    /// 无校验
    #[default]
    None = 0,
    /// 偶校验
    Even = 1,
    /// 奇校验
    Odd = 2,
}

impl Parity {
    /// 是否使能校验（PCE）
    #[inline]
    pub fn is_enable(self) -> bool {
        self != Parity::None
    }
}

/// 过采样
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum OverSampling {
    /// 16 倍过采样
    #[default]
    Sixteen = 0,
    /// 8 倍过采样
    Eight = 1,
}

impl OverSampling {
    /// 返回采样倍数（16 或 8）
    #[inline]
    pub fn div(&self) -> u32 {
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

/// 数据位长度
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum DataBits {
    /// 8 位数据
    #[default]
    Eight = 0,
    /// 9 位数据
    Nine = 1,
}

impl From<DataBits> for bool {
    #[inline]
    fn from(value: DataBits) -> Self {
        value == DataBits::Nine
    }
}

/// 硬件流控
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum HwFlowCtrl {
    /// 不使用硬件流控
    #[default]
    None,
    /// 仅 RTS
    Rts,
    /// 仅 CTS
    Cts,
    /// RTS + CTS
    RtsCts,
}

impl HwFlowCtrl {
    #[inline]
    pub fn rts(&self) -> bool {
        matches!(*self, HwFlowCtrl::Rts | HwFlowCtrl::RtsCts)
    }
    #[inline]
    pub fn cts(&self) -> bool {
        matches!(*self, HwFlowCtrl::Cts | HwFlowCtrl::RtsCts)
    }
}

/// 自动波特率检测模式（CR3.ABRMOD）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AutoBaudMode {
    /// 模式 0：从起始位开始测量，检测任意字符
    #[default]
    StartBit = 0,
    /// 模式 1：检测 RX 线上下降沿之间的时间
    FallingEdge = 1,
    /// 模式 2：检测 0x7F 帧
    Frame0x7F = 2,
    /// 模式 3：检测 0x55 帧
    Frame0x55 = 3,
}

/// IrDA 工作模式（CR3.IRLP）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum IrdaMode {
    /// 正常模式
    #[default]
    Normal = 0,
    /// 低功耗模式
    LowPower = 1,
}

/// 同步模式时钟输出极性（CR2.CPOL）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ClockPolarity {
    /// 空闲时 CK 为低电平
    #[default]
    IdleLow = 0,
    /// 空闲时 CK 为高电平
    IdleHigh = 1,
}

/// 同步模式时钟相位（CR2.CPHA）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ClockPhase {
    /// 第一个时钟沿进行数据捕获
    #[default]
    FirstEdge = 0,
    /// 第二个时钟沿进行数据捕获
    SecondEdge = 1,
}

/// 同步模式（CK 引脚）配置
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SyncClockConfig {
    /// 时钟极性
    pub polarity: ClockPolarity,
    /// 时钟相位
    pub phase: ClockPhase,
    /// 最后一个数据位是否输出时钟脉冲（LBCL）
    pub last_bit_clock: bool,
}

impl Default for SyncClockConfig {
    fn default() -> Self {
        Self {
            polarity: ClockPolarity::IdleLow,
            phase: ClockPhase::FirstEdge,
            last_bit_clock: false,
        }
    }
}

/// 接收唤醒方式（CR1.WAKE），仅在静默模式下有效
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum WakeMethod {
    /// 空闲线路唤醒
    #[default]
    IdleLine = 0,
    /// 地址标记唤醒
    AddressMark = 1,
}

/// 串口配置
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    /// 波特率
    pub baud_rate: Baud,
    /// 停止位
    pub stop_bit: StopBits,
    /// 校验方式
    pub parity: Parity,
    /// 数据位
    pub data_bits: DataBits,
    /// 过采样
    pub over_sampling: OverSampling,
    /// 硬件流控
    pub hw_flow_ctrl: HwFlowCtrl,
    /// 自动波特率检测；`Some(mode)` 时使能 ABR 并按该模式测量
    pub auto_baud: Option<AutoBaudMode>,
    /// 半双工（单线）模式，置位 CR3.HDSEL
    pub half_duplex: bool,
    /// IrDA 模式；`Some(mode)` 时使能 CR3.IREN
    pub irda: Option<IrdaMode>,
    /// 同步模式时钟输出；`Some(cfg)` 时使能 CR2.CLKEN
    pub sync_clock: Option<SyncClockConfig>,
    /// 静默模式（CR1.RWU）与唤醒方式（CR1.WAKE）
    pub mute: Option<WakeMethod>,
    /// 多处理器通信的节点地址（CR2.ADD，仅地址标记唤醒时使用）
    pub address: u8,
}

impl Default for Config {
    /// 默认 115200 Bd、8 位数据、无校验、1 停止位、16 倍过采样
    #[inline]
    fn default() -> Self {
        Self {
            baud_rate: 115200.Bd(),
            stop_bit: StopBits::One,
            parity: Parity::None,
            data_bits: DataBits::Eight,
            over_sampling: OverSampling::Sixteen,
            hw_flow_ctrl: HwFlowCtrl::None,
            auto_baud: None,
            half_duplex: false,
            irda: None,
            sync_clock: None,
            mute: None,
            address: 0,
        }
    }
}

impl Config {
    // ---- builder ----
    pub fn baud_rate(mut self, baud_rate: Baud) -> Self {
        self.baud_rate = baud_rate;
        self
    }
    pub fn stop_bit(mut self, stop_bit: StopBits) -> Self {
        self.stop_bit = stop_bit;
        self
    }
    pub fn parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }
    pub fn data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = data_bits;
        self
    }
    pub fn over_sampling(mut self, over_sampling: OverSampling) -> Self {
        self.over_sampling = over_sampling;
        self
    }
    pub fn hw_flow_ctrl(mut self, hw_flow_ctrl: HwFlowCtrl) -> Self {
        self.hw_flow_ctrl = hw_flow_ctrl;
        self
    }
    pub fn auto_baud(mut self, mode: AutoBaudMode) -> Self {
        self.auto_baud = Some(mode);
        self
    }
    pub fn half_duplex(mut self, en: bool) -> Self {
        self.half_duplex = en;
        self
    }
    pub fn irda(mut self, mode: IrdaMode) -> Self {
        self.irda = Some(mode);
        self
    }
    pub fn sync_clock(mut self, cfg: SyncClockConfig) -> Self {
        self.sync_clock = Some(cfg);
        self
    }
    pub fn mute(mut self, wake: WakeMethod) -> Self {
        self.mute = Some(wake);
        self
    }
    pub fn address(mut self, address: u8) -> Self {
        self.address = address;
        self
    }
}
