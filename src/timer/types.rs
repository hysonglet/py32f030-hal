use enumset::EnumSetType;

/// Timer error (current operations have no failure branch).
#[derive(Debug)]
pub enum Error {}

/// Unified APB timer identifier (used for clock index, interrupt vector, and async Waker array index).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerId {
    TIM1 = 0,
    TIM3 = 1,
    TIM14 = 2,
    TIM16 = 3,
    /// TIM17 is functionally identical to TIM16 and shares the same register layout.
    TIM17 = 4,
}

/// Input-capture and PWM output channels (unified enum; single-channel timers only have CH1).
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Channel {
    CH1,
    CH2,
    CH3,
    CH4,
}

impl Channel {
    /// Channel index (0-based).
    #[inline]
    pub fn index(self) -> usize {
        match self {
            Channel::CH1 => 0,
            Channel::CH2 => 1,
            Channel::CH3 => 2,
            Channel::CH4 => 3,
        }
    }

    /// Construct a channel from an index; returns None if out of range.
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
    /// Main output
    P,
    /// Complementary output (only TIM1 / TIM16 support it)
    N,
}

/// Channel type
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ChannelType {
    /// Channel configured as output
    Pwm = 0,
    /// Channel configured as input, ICx mapped to TIx
    Capture1 = 1,
    /// Channel configured as input, ICx mapped to TIy
    Capture2 = 2,
    /// Channel configured as input, ICx mapped to TRC
    Capture3 = 3,
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum ChannelMode {
    /// Frozen: the CCRx vs CNT comparison has no effect on OCxREF
    #[default]
    Mode0 = 0,
    /// Force OCxREF high on match
    Mode1 = 1,
    /// Force OCxREF low on match
    Mode2 = 2,
    /// Toggle OCxREF on match
    Mode3 = 3,
    /// Force to inactive level
    Mode4 = 4,
    /// Force to active level
    Mode5 = 5,
    /// PWM mode 1
    PWM1 = 6,
    /// PWM mode 2
    PWM2 = 7,
}

/// Counting direction
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum CountDirection {
    /// Count up
    Up = 0,
    /// Count down
    Down = 1,
}

/// Center-aligned mode
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CenterAlignedMode {
    /// Edge-aligned
    EdgeAligned = 0,
    /// Center-aligned mode 1
    CenterAligned1 = 1,
    /// Center-aligned mode 2
    CenterAligned2 = 2,
    /// Center-aligned mode 3
    CenterAligned3 = 3,
}

/// Software trigger events (EGR)
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

/// Timer events (unified enum; unsupported events are always false / ignored on a given timer).
#[derive(EnumSetType, Debug)]
pub enum Event {
    /// Update interrupt flag
    UIF,
    /// Capture/compare 1 interrupt flag
    CC1IF,
    /// Capture/compare 2 interrupt flag
    CC2IF,
    /// Capture/compare 3 interrupt flag
    CC3IF,
    /// Capture/compare 4 interrupt flag
    CC4IF,
    /// COM interrupt flag
    COMIF,
    /// Trigger interrupt flag
    TIF,
    /// Break interrupt flag
    BIF,
    /// Capture/compare 1 over-capture flag
    CC1OF,
    /// Capture/compare 2 over-capture flag
    CC2OF,
    /// Capture/compare 3 over-capture flag
    CC3OF,
    /// Capture/compare 4 over-capture flag
    CC4OF,
}

/// Clock prescaler factor (tDTS)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClockDiv {
    /// tDTS = tCK_INT
    Div1 = 0,
    /// tDTS = 2 x tCK_INT
    Div2 = 1,
    /// tDTS = 4 x tCK_INT
    Div4 = 2,
}

/// Input capture polarity
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputCapturePolarity {
    /// Capture on rising edge
    Rising = 0,
    /// Capture on falling edge
    Falling = 1,
    /// Capture on both edges
    BothEdge = 2,
}

/// Input capture selection (CCxS)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputCaptureSelection {
    /// Direct: ICx mapped to TIx
    DirectTI = 1,
    /// Indirect: ICx mapped to TIy
    IndirectTI = 2,
    /// ICx mapped to TRC
    TRC = 3,
}

/// Input capture prescaler
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputCapturePrescaler {
    Div1 = 0,
    Div2 = 1,
    Div4 = 2,
    Div8 = 3,
}

/// Input capture digital filter
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

/// Encoder mode
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EncoderMode {
    /// Count on TI1 edges only (x2)
    Ti1 = 1,
    /// Count on TI2 edges only (x2)
    Ti2 = 2,
    /// Count on both TI1 and TI2 edges (x4)
    Ti12 = 3,
}

/// Slave mode selection (SMS)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlaveMode {
    /// Slave mode disabled
    Disable = 0,
    /// Encoder mode 1
    EncoderTi1 = 1,
    /// Encoder mode 2
    EncoderTi2 = 2,
    /// Encoder mode 3
    EncoderTi12 = 3,
    /// Reset mode
    Reset = 4,
    /// Gated mode
    Gated = 5,
    /// Trigger mode
    Trigger = 6,
    /// External clock mode 1 (only TIM1 has ETR)
    ExternalClock1 = 7,
}

/// Trigger input selection (TS)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TriggerSelection {
    ITR0 = 0,
    ITR1 = 1,
    ITR2 = 2,
    ITR3 = 3,
    /// TI1 edge detector
    TI1F_ED = 4,
    /// Filtered TI1FP1
    TI1FP1 = 5,
    /// Filtered TI2FP2
    TI2FP2 = 6,
    /// External trigger (only TIM1 supports it)
    ETRF = 7,
}

/// Master mode trigger output (TRGO, MMS)
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

/// OCREF clear source
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OcRefClearSource {
    /// Use ETR as the OCREF_CLR input
    Etc = 1,
    /// Not used
    None = 0,
}

/// Dead time (DTG)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DeadTime(pub u8);

impl DeadTime {
    pub fn value(&self) -> u8 {
        self.0
    }
}

/// Lock level
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LockLevel {
    Off = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
}

/// Output idle/break behavior
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OffState {
    /// Output disabled
    Disable = 0,
    /// Output enabled, determined by idle state
    Idle = 1,
}

/// Break polarity
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BreakPolarity {
    /// Active low
    Low = 0,
    /// Active high
    High = 1,
}

/// TIM14 TI1 input remap (TIM14_OR)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ti1Remap {
    /// TI1 comes from the TIM14_CH1 pin
    Channel1 = 0,
    /// TI1 comes from the alternate input
    Alternative = 1,
}

/// One-pulse mode
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OnePulseMode {
    /// Single: the counter stops after an update event (OPM=1)
    Single = 1,
    /// Repetitive: the counter keeps running (OPM=0)
    Repetitive = 0,
}


/// Channel output configuration (polarity / enable / idle state)
#[derive(Default, Clone, Copy)]
pub struct ChannelOutputConfig {
    /// Output enable
    pub state: bool,
    /// Active polarity: true means active-high
    pub polarity: bool,
    /// Idle state (OIS, only valid for TIM1 / TIM16)
    pub idle_state: bool,
}

/// Channel configuration (output compare / PWM)
#[derive(Default, Clone, Copy)]
pub struct ChannelConfig {
    /// Channel mode
    pub mode: ChannelMode,
    /// Whether external events may clear OCREF
    pub clear: bool,
    /// Output compare fast enable
    pub fast: bool,
    /// Output compare preload enable
    pub preload: bool,
    /// Capture/compare value (duty cycle)
    pub compare: u16,
    /// Main output (CHx) configuration
    pub ch: Option<ChannelOutputConfig>,
    /// Complementary output (CHxN) configuration, only valid for TIM1 / TIM16
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
    /// Set the main output configuration.
    pub fn ch(mut self, ch: ChannelOutputConfig) -> Self {
        self.ch = Some(ch);
        self
    }
    /// Set the complementary output configuration (only TIM1 / TIM16).
    pub fn n_ch(mut self, n_ch: ChannelOutputConfig) -> Self {
        self.n_ch = Some(n_ch);
        self
    }
}

/// Break and dead-time configuration (only TIM1 / TIM16)
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
