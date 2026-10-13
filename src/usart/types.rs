use embedded_time::rate::{Baud, Extensions};

/// USART errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Start-bit timeout.
    StartTimeout,
    /// Read timeout.
    ReadTimeout,
    /// Write timeout.
    WriteTimeout,
    /// DMA transfer error (or a DMA interface used without a DMA channel configured).
    DMA,
    /// Noise detected.
    Noise,
    /// Framing error.
    Frame,
    /// Parity error.
    Parity,
    /// Overrun error.
    Overrun,
    /// Other error.
    Others,
}

/// Stop bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum StopBits {
    /// 1 stop bit.
    #[default]
    One = 0,
    /// 2 stop bits.
    Two = 1,
}

/// Parity mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Parity {
    /// No parity.
    #[default]
    None = 0,
    /// Even parity.
    Even = 1,
    /// Odd parity.
    Odd = 2,
}

impl Parity {
    /// Whether parity is enabled (PCE).
    #[inline]
    pub fn is_enable(self) -> bool {
        self != Parity::None
    }
}

/// Oversampling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum OverSampling {
    /// 16x oversampling.
    #[default]
    Sixteen = 0,
    /// 8x oversampling.
    Eight = 1,
}

impl OverSampling {
    /// Return the oversampling factor (16 or 8).
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

/// Data bit length.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum DataBits {
    /// 8 data bits.
    #[default]
    Eight = 0,
    /// 9 data bits.
    Nine = 1,
}

impl From<DataBits> for bool {
    #[inline]
    fn from(value: DataBits) -> Self {
        value == DataBits::Nine
    }
}

/// Hardware flow control.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum HwFlowCtrl {
    /// No hardware flow control.
    #[default]
    None,
    /// RTS only.
    Rts,
    /// CTS only.
    Cts,
    /// RTS + CTS.
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

/// Auto-baud-rate detection mode (CR3.ABRMOD).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AutoBaudMode {
    /// Mode 0: measure from the start bit, detect any character.
    #[default]
    StartBit = 0,
    /// Mode 1: measure the time between falling edges on the RX line.
    FallingEdge = 1,
    /// Mode 2: detect a 0x7F frame.
    Frame0x7F = 2,
    /// Mode 3: detect a 0x55 frame.
    Frame0x55 = 3,
}

/// IrDA operating mode (CR3.IRLP).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum IrdaMode {
    /// Normal mode.
    #[default]
    Normal = 0,
    /// Low-power mode.
    LowPower = 1,
}

/// Synchronous-mode clock output polarity (CR2.CPOL).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ClockPolarity {
    /// CK idle low.
    #[default]
    IdleLow = 0,
    /// CK idle high.
    IdleHigh = 1,
}

/// Synchronous-mode clock phase (CR2.CPHA).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ClockPhase {
    /// Data captured on the first clock edge.
    #[default]
    FirstEdge = 0,
    /// Data captured on the second clock edge.
    SecondEdge = 1,
}

/// Synchronous-mode (CK pin) configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SyncClockConfig {
    /// Clock polarity.
    pub polarity: ClockPolarity,
    /// Clock phase.
    pub phase: ClockPhase,
    /// Whether to output a clock pulse on the last data bit (LBCL).
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

/// Receive wake-up method (CR1.WAKE), valid only in mute mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum WakeMethod {
    /// Idle-line wake-up.
    #[default]
    IdleLine = 0,
    /// Address-mark wake-up.
    AddressMark = 1,
}

/// USART configuration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    /// Baud rate.
    pub baud_rate: Baud,
    /// Stop bits.
    pub stop_bit: StopBits,
    /// Parity mode.
    pub parity: Parity,
    /// Data bits.
    pub data_bits: DataBits,
    /// Oversampling.
    pub over_sampling: OverSampling,
    /// Hardware flow control.
    pub hw_flow_ctrl: HwFlowCtrl,
    /// Auto-baud-rate detection; `Some(mode)` enables ABR and measures in that mode.
    pub auto_baud: Option<AutoBaudMode>,
    /// Half-duplex (single-wire) mode, sets CR3.HDSEL.
    pub half_duplex: bool,
    /// IrDA mode; `Some(mode)` enables CR3.IREN.
    pub irda: Option<IrdaMode>,
    /// Synchronous-mode clock output; `Some(cfg)` enables CR2.CLKEN.
    pub sync_clock: Option<SyncClockConfig>,
    /// Mute mode (CR1.RWU) and wake-up method (CR1.WAKE).
    pub mute: Option<WakeMethod>,
    /// Node address for multiprocessor communication (CR2.ADD, used only with address-mark wake-up).
    pub address: u8,
}

impl Default for Config {
    /// Defaults: 115200 Bd, 8 data bits, no parity, 1 stop bit, 16x oversampling.
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
