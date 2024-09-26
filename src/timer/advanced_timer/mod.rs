mod counter;
mod future;
mod hal;
mod pins;
mod pwm;

use core::{marker::PhantomData, u16};
use enumset::EnumSetType;

pub use counter::Counter;
pub use pwm::Pwm;

use crate::{
    clock::peripheral::{PeripheralClockIndex, PeripheralEnable, PeripheralInterrupt},
    mode::Mode,
};
use embassy_hal_internal::{into_ref, Peripheral};

pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

/// 高级定时器
#[derive(PartialEq)]
pub enum AdvancedTimer {
    TIM1,
}

impl PeripheralEnable for AdvancedTimer {
    fn clock(&self, en: bool) {
        match *self {
            Self::TIM1 => PeripheralClockIndex::TIM1.clock(en),
        }
    }

    fn is_open(&self) -> bool {
        match *self {
            Self::TIM1 => PeripheralClockIndex::TIM1.is_open(),
        }
    }

    fn reset(&self) {
        match *self {
            Self::TIM1 => PeripheralClockIndex::TIM1.reset(),
        }
    }
}

impl PeripheralInterrupt for AdvancedTimer {
    fn interrupt(&self) -> crate::pac::interrupt {
        match *self {
            Self::TIM1 => crate::pac::interrupt::TIM1_BRK_UP_TRG_COM,
        }
    }
}

/// 输入捕获和输出pwm通道
#[derive(PartialEq)]
pub enum Channel {
    CH1,
    CH1_N,
    CH2,
    CH2_N,
    CH3,
    CH3_N,
    CH4,
}

/// 通道类型
#[derive(PartialEq)]
pub enum ChannelType {
    /// 通道被配置为输出；
    Pwm = 0,
    /// 通道被配置为输入， IC3 映射在 TI3 上
    Capture1 = 1,
    /// 通道被配置为输入， IC3 映射在 TI4 上；
    Capture2 = 2,
    /// 通道被配置为输入， IC3 映射在 TRC 上。此模式仅工作在内部触发器输入被选中时
    Capture3 = 3,
}

#[derive(PartialEq, Eq)]
pub enum ChannelOutputMode {
    /// 输出比较 1 模式
    /// 该位定义了输出参考信号 OC1REF 的动作，而 OC1REF决定了 OC1、 OC1N 的值。 OC1REF 是高电平有效，
    /// 而OC1、 OC1N 的有效电平取决于 CC1P、 CC1NP 位。000：冻结。输出比较寄存器 TIM1_CCR1 与计数器
    /// TIM1_CNT 间的比较对 OC1REF 不起作用
    Mode0 = 0,
    /// 匹配时设置通道1为有效电平。当计数器TIMx_CNT的值与捕获/比较寄存器1(TIMx_CCR1)相同时，
    /// 强制 OC1REF 为高。
    Mode1 = 1,
    /// 010：匹配时设置通道1为无效电平。当计数器TIMx_CNT的值与捕获/比较寄存器1(TIMx_CCR1)相同时，
    /// 强制 OC1REF 为低。
    Mode2 = 2,
    /// 翻转。当 TIM1_CCR1=TIM1_CNT 时，翻转OC1REF 的电平。
    Mode3 = 3,
    /// 强制为无效电平。强制 OC1REF 为低。
    Mode4 = 4,
    /// 强制为有效电平。强制 OC1REF 为高。
    Mode5 = 5,
    /// PWM 模式 1－在向上计数时，一旦TIM1_CNT<TIM1_CCR1 时通道 1 为有效电平，否则为无效电平；在向下计数时，
    /// 一旦TIM1_CNT>TIM1_CCR1 时通道1为无效电平(OC1REF=0)，否则为有效电平(OC1REF=1)。
    Mode6 = 6,
    /// PWM 模式 2－ 在向上计数时，一旦TIM1_CNT<TIM1_CCR1 时通道 1 为无效电平，否则为有效电平；在向下计数时，
    /// 一旦 TIM1_CNT>TIM1_CCR1 时通道 1 为有效电平，否则为无效电平。
    Mode7 = 7,
}

#[derive(Debug)]
pub enum Error {}

/// 记数模式
#[derive(PartialEq, Clone, Copy)]
pub enum CountDirection {
    /// 向上计数模式，是从 0 到自动装载值的计数器，然后又从 0 重新开始计数，并产生一个计数的溢出事件。
    /// 如果重复计数器被使用，则在向上计数器重复几次（对重复计数器可编程）后，产生更新事件。否则，在每个计数溢出时，
    /// 产生更新事件。
    Up = 0,
    /// 向下计数模式，从自动装载的值开始向下计数到 0，然后重新开始从自动装载的值向下计数，并产生一个向下溢出事件。
    /// 如果使用了重复计数器，当向下计数重复了重复计数寄存器(TIMx_RCR)中设定的次数后，将产生更新事件(UEV)，否则每次
    /// 计数器下溢时才产生更新事件。
    Down = 1,
}

impl From<CountDirection> for bool {
    fn from(value: CountDirection) -> Self {
        match value {
            CountDirection::Down => false,
            CountDirection::Up => true,
        }
    }
}

///时钟分频因子
/// 这 2 位定义在定时器时钟(CK_INT)频率，死区时间和由死区发生器与数字滤波器(ETR,Tix)所用的采样时钟之间的分频比例
#[derive(PartialEq, Clone, Copy)]
pub enum ClockDiv {
    ///  tDTS = tCK_INT
    DIV1,
    /// tDTS = 2 x tCK_INT
    DIV2,
    /// tDTS = 4 x tCK_INT
    DIV4,
}

/// 选择中央对齐模式
///     注：在计数器开启时(CEN=1)，不允许从边沿对齐模式转换到中央对齐模式
#[derive(Clone, Copy, PartialEq)]
pub enum CenterAlignedMode {
    /// 边沿对齐模式。计数器依据方向位(DIR)向上或向下计数。
    EdgeAligned = 0,
    /// 中央对齐模式 1。计数器交替地向上和向下计数。配置为输出的通道(TIM3_CCMRx 寄存器中 CCxS=00)的输出比较中断标
    /// 志位，只在计数器向下计数时被设置。
    CenterAligned1 = 1,
    /// 中央对齐模式 2。计数器交替地向上和向下计数。计数器交替地向上和向下计数。配置为输出的通道(TIM3_CCMRx 寄存器
    /// 中 CCxS=00)的输出比较中断标志位，只在计数器向上计数时被设置。
    CenterAligned2 = 2,
    /// 央对齐模式 3。计数器交替地向上和向下计数。计数器交替地向上和向下计数。配置为输出的通道(TIM3_CCMRx 寄存器中
    ///  CCxS=00)的输出比较中断标志位，在计数器向上和向下计数时均被设置。
    CenterAligned3 = 3,
}

pub struct Capture;
pub struct Hall;
pub struct Motor;

macro_rules! impl_sealed_timer {
    (
        $peripheral: ident, $timer_id: ident
    ) => {
        impl hal::sealed::Instance for crate::mcu::peripherals::$peripheral {
            fn id() -> AdvancedTimer {
                AdvancedTimer::$timer_id
            }
        }
        impl Instance for crate::mcu::peripherals::$peripheral {}
    };
}

pub struct AnyTimer<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

impl_sealed_timer!(TIM1, TIM1);

impl<'d, T: Instance, M: Mode> AnyTimer<'d, T, M> {
    /// 新建一个 timer
    pub fn new(_timer: impl Peripheral<P = T>) -> Result<Self, Error> {
        into_ref!(_timer);

        // 开启外设时钟
        // T::id().reset();
        T::id().open();

        Ok(Self {
            _t: PhantomData,
            _m: PhantomData,
        })
    }

    /// 返回定时器外设的时钟
    pub fn get_timer_clock() -> u32 {
        T::get_time_pclk()
    }

    /// 转换成计数模式
    pub fn as_counter(self) -> Counter<'d, T, M> {
        Counter::new()
    }

    /// 转换成pwm模式
    pub fn as_pwm(
        self,
        channel_1_pin: Option<impl Peripheral<P = impl TimerChannel1Pin<T>> + 'd>,
        channel_1_n_pin: Option<impl Peripheral<P = impl TimerChannel1NPin<T>> + 'd>,
        channel_2_pin: Option<impl Peripheral<P = impl TimerChannel2Pin<T>> + 'd>,
        channel_2_n_pin: Option<impl Peripheral<P = impl TimerChannel2NPin<T>> + 'd>,
        channel_3_pin: Option<impl Peripheral<P = impl TimerChannel3Pin<T>> + 'd>,
        channel_3_n_pin: Option<impl Peripheral<P = impl TimerChannel3NPin<T>> + 'd>,
        channel_4_pin: Option<impl Peripheral<P = impl TimerChannel4Pin<T>> + 'd>,
    ) -> Pwm<'d, T> {
        Pwm::new(
            channel_1_pin,
            channel_1_n_pin,
            channel_2_pin,
            channel_2_n_pin,
            channel_3_pin,
            channel_3_n_pin,
            channel_4_pin,
        )
    }
}

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
    /// COM 中断标记一旦产生 COM 事件（当 CcxE、 CcxNE、 OCxM 已被更新）该位由硬件置 1。它由软件清 0。
    COMIF,
    /// 触发器中断标记当发生触发事件（当从模式控制器处于除门控模式外的其它模式时,在 TRGI 输入端检测到有效边沿，
    /// 或或门控模式下的任一边沿）时由硬件对该位置。它由软件清 0。
    TIF,
    /// 刹车中断标记一旦刹车输入有效，由硬件对该位置 1。如果刹车输入无效，则该位可由软件清 0。
    BIF,
    /// 捕获/比较 1 过捕获标记仅当相应的通道被配置为输入捕获时，该标记可由硬件置1。写 0 可清除该位。
    CC1OF,
    /// 捕获/比较 2 过捕获标记
    CC2OF,
    /// 捕获/比较 3 过捕获标记
    CC3OF,
    /// 捕获/比较 4 过捕获标记
    CC4OF,
}

// 定义一个 定时器引脚 的trait
pin_af_for_instance_def!(TimerChannel1Pin, Instance);
pin_af_for_instance_def!(TimerChannel1NPin, Instance);
pin_af_for_instance_def!(TimerChannel2Pin, Instance);
pin_af_for_instance_def!(TimerChannel2NPin, Instance);
pin_af_for_instance_def!(TimerChannel3Pin, Instance);
pin_af_for_instance_def!(TimerChannel3NPin, Instance);
pin_af_for_instance_def!(TimerChannel4Pin, Instance);
pin_af_for_instance_def!(TimerBkInPin, Instance);
pin_af_for_instance_def!(TimerEtrPin, Instance);
