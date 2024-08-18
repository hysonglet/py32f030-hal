mod counter;
mod hal;
use core::{marker::PhantomData, u16};

pub use counter::Counter;
use embassy_hal_internal::{into_ref, Peripheral};

use crate::{
    clock::peripheral::{PeripheralClockIndex, PeripheralEnable},
    mode::Mode,
};

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

/// 输入捕获和输出pwm通道
#[derive(PartialEq)]
pub enum Channel {
    CH1,
    CH2,
    CH3,
    CH4,
}

#[derive(Debug)]
pub enum Error {}

/// 记数模式
#[derive(PartialEq, Clone, Copy)]
pub enum CountDirection {
    /// 向上计数模式，是从 0 到自动装载值的计数器，然后又从 0 重新开始计数，并产生一个计数的溢出事件。
    /// 如果重复计数器被使用，则在向上计数器重复几次（对重复计数器可编程）后，产生更新事件。否则，在每个计数溢出时，产生更新事件。
    Up = 0,
    /// 向下计数模式，从自动装载的值开始向下计数到 0，然后重新开始从自动装载的值向下计数，并产生一个向下溢出事件。
    /// 如果使用了重复计数器，当向下计数重复了重复计数寄存器(TIMx_RCR)中设定的次数后，将产生更新事件(UEV)，否则每次计数器下溢时才产生更新事件。
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
    /// 中央对齐模式 1。计数器交替地向上和向下计数。配置为输出的通道(TIM3_CCMRx 寄存器中 CCxS=00)的输出比较中断标志位，只在计数器向下计数时被设置。
    CenterAligned1 = 1,
    /// 中央对齐模式 2。计数器交替地向上和向下计数。计数器交替地向上和向下计数。配置为输出的通道(TIM3_CCMRx 寄存器中 CCxS=00)的输出比较中断标志位，只在计数器向上计数时被设置。
    CenterAligned2 = 2,
    /// 央对齐模式 3。计数器交替地向上和向下计数。计数器交替地向上和向下计数。配置为输出的通道(TIM3_CCMRx 寄存器中 CCxS=00)的输出比较中断标志位，在计数器向上和向下计数时均被设置。
    CenterAligned3 = 3,
}

/// 定时器基本配置
#[derive(Clone, Copy)]
pub struct BaseConfig {
    count_direction: CountDirection,
    center_aligned_mode: CenterAlignedMode,
    prescaler: u16,
    // period: u16,
    // auto_reload: u16,
    // repetition: Option<u16>, // clock_div: ClockDiv,
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            count_direction: CountDirection::Up,
            center_aligned_mode: CenterAlignedMode::EdgeAligned,
            prescaler: u16::MAX,
            // period: u16::MAX,
            // auto_reload: u16::MAX,
            // repetition: None,
            // clock_div: ClockDiv::DIV1,
        }
    }
}

impl BaseConfig {
    pub fn count_direction(self, direction: CountDirection) -> Self {
        Self {
            count_direction: direction,
            ..self
        }
    }

    pub fn center_aligned_mode(self, mode: CenterAlignedMode) -> Self {
        Self {
            center_aligned_mode: mode,
            ..self
        }
    }

    pub fn prescaler(self, prescaler: u16) -> Self {
        Self { prescaler, ..self }
    }

    // pub fn auto_reload(self, auto_reload: Option<u16>) -> Self {
    //     Self {
    //         auto_reload,
    //         ..self
    //     }
    // }

    // pub fn period(self, period: u16) -> Self {
    //     Self { period, ..self }
    // }

    // pub fn clock_div(self, clock_div: ClockDiv) -> Self {
    //     Self { clock_div, ..self }
    // }
}

impl BaseConfig {
    // pub fn set_period_reload(self, period: u16, reload: u16) -> Self {
    //     let config = Self::default();
    //     config.period(period).auto_reload(Some(reload))
    // }

    // pub fn set_delay(&mut self, us: u32) -> Result<(), Error> {
    //     //timer_cnt_freq = Hplk/div;

    //     todo!()
    // }
}

pub struct Capture;
pub struct Pwm;
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
    pub fn new_inner(config: BaseConfig) -> Result<(), Error> {
        // 开启外设时钟
        T::id().open();
        // 将配置写到外设
        T::base_config(config)
    }

    /// 新建一个 timer
    pub fn new(_timer: impl Peripheral<P = T>) -> Result<Self, Error> {
        into_ref!(_timer);

        // Self::new_inner(config)?;
        Ok(Self {
            _t: PhantomData,
            _m: PhantomData,
        })
    }

    /// 返回定时器外设的时钟
    pub fn get_timer_clock() -> u32 {
        T::get_time_pclk()
    }

    // 基本的配置
    pub fn base_config(&self, config: BaseConfig) -> Result<(), Error> {
        T::base_config(config)
    }

    /// 设置预分配，分频值： (prescaler + 1)
    pub fn set_prescaler(&self, prescaler: u16) {
        T::set_prescaler(prescaler)
    }

    pub fn set_period(&self, period: u16) {
        T::set_period_cnt(period)
    }

    pub fn set_reload(&self, reload: u16, buff: bool) {
        T::set_auto_reload(reload);
        T::enable_auto_reload_buff(buff);
    }

    /// 转换成计数模式
    pub fn as_counter(self) -> Counter<'d, T, M> {
        let prescaler = (Self::get_timer_clock() / 1_000_000 - 1) as u16;
        // 设置默认clk时钟为1M，即1us
        assert!(prescaler < u16::MAX);

        let config = BaseConfig::default().prescaler(prescaler);
        let _ = Self::new_inner(config);

        Counter::new()
    }
}
