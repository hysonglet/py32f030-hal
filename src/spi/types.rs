use crate::clock;
use embedded_hal::spi::{Mode, Phase, Polarity};

/// SPI 错误
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Error {
    /// 初始化失败
    Init,
    /// 读失败
    Read,
    /// 写失败
    Write,
    /// 等待超时
    Timeout,
    /// 总线忙
    Busy,
    /// 溢出（OVR）
    Overrun,
    /// 模式错误（MODF，NSS 被拉低）
    ModeFault,
    /// DMA 传输错误（或未配置 DMA 通道却使用 DMA 接口）
    Dma,
}

/// 双向数据模式
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum BidirectionalMode {
    /// 双线单向
    #[default]
    Line2Unidirectional = 0,
    /// 单线双向
    Line1Bidirectional = 1,
}

/// 帧格式（MSB / LSB 先发）
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum BitOrder {
    /// 高位先发
    #[default]
    MSB = 0,
    /// 低位先发
    LSB = 1,
}

/// 波特率分频
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
    /// 返回实际波特率
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

/// 主从角色
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum Rule {
    /// 从机
    Slave = 0,
    /// 主机
    #[default]
    Master = 1,
}

/// 从机速度模式
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum SlaveSpeedMode {
    /// 普通模式（最快 pclk/4 以下）
    #[default]
    Normal = 0,
    /// 快速模式
    Fast = 1,
}

/// 数据帧长度
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum DataLength {
    /// 8 位
    #[default]
    Length8 = 0,
    /// 16 位
    Length16 = 1,
}

/// SPI 配置
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// 时钟极性与相位（CPOL/CPHA）
    pub mode: Mode,
    /// 帧格式（MSB / LSB 先发）
    pub bit_order: BitOrder,
    /// 波特率分频
    pub baud_rate_div: BaudRateDiv,
    /// 数据帧长度（8 / 16 位）
    pub data_len: DataLength,
    /// 双向数据模式；单线双向时 MOSI 兼作收发
    pub bidirectional: BidirectionalMode,
    /// 只接收（关闭输出，RXONLY）
    pub rx_only: bool,
    /// 从机速度模式
    pub slave_speed_mode: SlaveSpeedMode,
    /// 使能 SS 输出（SSOE），主机模式下由硬件驱动 NSS
    pub ss_output_enable: bool,
    /// 软件从设备管理（SSM）。未接 NSS 引脚时必须为 true，否则会误报 MODF
    pub software_slave_management: bool,
    /// 接收 FIFO 阈值：true 表示 8 位即产生 RXNE，false 表示 16 位
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
