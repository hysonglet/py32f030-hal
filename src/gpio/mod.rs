//! General purpose input/output (GPIO) driver.
//!
//! Output
//!
//! ```rust, ignore
//! let p = hal::init(Default::default());
//! let gpioa = p.GPIOA.split();
//! let mut led = Output::new(gpioa.PA10, PinIoType::PullDown, PinSpeed::Low);
//! ```
//!
//! Input
//! ```rust, ignore
//! let p = hal::init(Default::default());
//! let gpioa = p.GPIOA.split();
//! let key = Input::new(gpioa.PA12, PinPullUpDown::PullUp, PinSpeed::Low);
//! ```
//!
//! AF
//! ```rust, ignore
//! let gpioa = p.GPIOA.split();
//! let _mco_pin = Af::new(
//!    gpioa.PA1,
//!    PinAF::AF15,
//!    PinSpeed::VeryHigh,
//!    PinIoType::PullUp,
//! );
//! Mco::select(clock::McoSelect::SysClk, clock::McoDIV::DIV1);
//! ```

pub(crate) mod hal;
mod types;

use hal::*;

pub use types::*;

use crate::clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex};
use embassy_hal_internal::{impl_peripheral, into_ref, Peripheral, PeripheralRef};

/// GPIO Port Index
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GpioPort {
    GPIOA = 0,
    GPIOB = 1,
    GPIOF = 2,
}

impl PeripheralIdToClockIndex for GpioPort {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            GpioPort::GPIOA => PeripheralClockIndex::GPIOA,
            GpioPort::GPIOB => PeripheralClockIndex::GPIOB,
            GpioPort::GPIOF => PeripheralClockIndex::GPIOF,
        }
    }
}

impl From<usize> for GpioPort {
    fn from(value: usize) -> Self {
        match value {
            0 => GpioPort::GPIOA,
            1 => GpioPort::GPIOB,
            2 => GpioPort::GPIOF,
            _ => unreachable!(),
        }
    }
}

/// AnyPin
pub struct AnyPin {
    port_pin: u8,
}

impl_peripheral!(AnyPin);
impl sealed::Pin for AnyPin {
    fn port_pin(&self) -> u8 {
        self.port_pin
    }
}

impl Pin for AnyPin {
    fn degrade(self) -> AnyPin {
        self
    }
}

/// Flexible GPIO pin.
///
/// Flex 接口的 pin
/// 该结构体不对 api 调用做任何保护，例如引脚调用输出配置后，再读取，也许硬件不支持这种使用，但
/// 这个对象依旧不阻止调用，如果希望更强的保护，推荐使用 `Input,Output,Analog` 等对象，保证
/// 接口不被乱用
pub struct Flex<'d> {
    pub(crate) pin: PeripheralRef<'d, AnyPin>,
}

pub trait Pin: Peripheral<P = Self> + Into<AnyPin> + sealed::Pin + Sized + 'static {
    #[inline]
    fn degrade(self) -> AnyPin {
        AnyPin {
            port_pin: self.port_pin(),
        }
    }
}

use sealed::Pin as _;

impl<'d> Flex<'d> {
    #[inline]
    pub fn new(pin: impl Peripheral<P = impl Pin> + 'd) -> Self {
        into_ref!(pin);
        Self {
            pin: pin.map_into(),
        }
    }

    #[inline]
    pub fn port(&self) -> GpioPort {
        self.pin.port()
    }

    #[inline]
    pub fn pin(&self) -> usize {
        self.pin.pin()
    }

    /// Put the pin into input mode.
    #[inline]
    pub fn set_as_input(&self, pull: PinPullUpDown, speed: PinSpeed) {
        critical_section::with(|_| {
            self.pin.set_mode(PinMode::Input);
            self.pin.set_push_pull(pull);
            self.pin.set_speed(speed);
        });
    }

    /// Put the pin into output mode.
    #[inline]
    pub fn set_as_output(&self, io_type: PinIoType, speed: PinSpeed) {
        critical_section::with(|_| {
            self.pin.set_mode(PinMode::Output);
            self.pin.set_io_type(io_type);
            self.pin.set_speed(speed);
        });
    }

    /// Put the pin into analog mode.
    #[inline]
    pub fn set_as_analog(&self) {
        critical_section::with(|_| {
            self.pin.set_mode(PinMode::Analog);
        });
    }

    /// Put the pin into alternate function mode.
    #[inline]
    pub fn set_as_af(&self, af: PinAF, speed: PinSpeed, io_type: PinIoType) {
        critical_section::with(|_| {
            self.pin.set_mode(PinMode::Af);
            self.pin.set_af(af);
            self.pin.set_speed(speed);
            self.pin.set_io_type(io_type);
        });
    }

    /// Set internal pull-up or pull-down configuration for this pin.
    #[inline]
    pub fn set_push_pull(&self, push_pull: PinPullUpDown) {
        self.pin.set_push_pull(push_pull);
    }

    /// Set open-drain or push-pull output type for this pin.
    #[inline]
    pub fn set_open_drain(&self, open_drain: PinOutputType) {
        self.pin.set_output_type(open_drain);
    }

    /// Set the I/O type of current pin.
    #[inline]
    pub fn set_io_type(&self, io_type: PinIoType) {
        self.pin.set_io_type(io_type)
    }

    /// Read the input electrical level of the current pin.
    #[inline]
    pub fn read(&self) -> PinLevel {
        self.pin.read()
    }

    /// Write an output electrical level into the current pin.
    #[inline]
    pub fn write(&self, level: PinLevel) {
        self.pin.write(level)
    }
}

impl AnyPin {
    /// # Safety
    #[inline]
    pub unsafe fn steal(port: GpioPort, pin: u8) -> Self {
        assert!(pin < 16);
        // safe
        let port_pin = ((port as u8) << 4) | pin;
        Self { port_pin }
    }
}

/// Input-mode driver for a GPIO pin.
pub struct Input<'d> {
    pub(crate) pin: Flex<'d>,
}

/// Output-mode driver for a GPIO pin.
pub struct Output<'d> {
    pub(crate) pin: Flex<'d>,
}

/// Alternate function mode driver for a GPIO pin.
pub struct Af<'d> {
    pub(crate) _pin: Flex<'d>,
}

/// Analog-mode driver for a GPIO pin.
pub struct Analog<'d> {
    pub(crate) _pin: Flex<'d>,
}

impl<'d> Input<'d> {
    /// Create a GPIO input driver for a pin with the provided pull configuration.
    #[inline]
    pub fn new(
        pin: impl Peripheral<P = impl Pin> + 'd,
        pull: PinPullUpDown,
        speed: PinSpeed,
    ) -> Self {
        let pin = Flex::new(pin);

        pin.set_as_input(pull, speed);

        Self { pin }
    }

    /// Read the input electrical level of the current pin.
    #[inline]
    pub fn read(&self) -> PinLevel {
        self.pin.read()
    }
}

impl<'d> Output<'d> {
    /// Create a GPIO output driver for a pin with the provided I/O type and speed configurations.
    #[inline]
    pub fn new(
        pin: impl Peripheral<P = impl Pin> + 'd,
        io_type: PinIoType,
        speed: PinSpeed,
    ) -> Self {
        let pin = Flex::new(pin);

        pin.set_as_output(io_type, speed);

        Self { pin }
    }

    /// Read the input electrical level of the current pin.
    #[inline]
    pub fn read(&self) -> PinLevel {
        self.pin.read()
    }

    /// Write an output electrical level into the current pin.
    #[inline]
    pub fn write(&self, level: PinLevel) {
        self.pin.write(level)
    }
}

impl<'d> Af<'d> {
    /// Create a GPIO alternate function driver for a pin with the provided alternate function,
    /// I/O type and speed configurations.
    #[inline]
    pub fn new(
        pin: impl Peripheral<P = impl Pin> + 'd,
        af: impl Into<PinAF>,
        speed: PinSpeed,
        io_type: PinIoType,
    ) -> Self {
        let pin = Flex::new(pin);

        pin.set_as_af(af.into(), speed, io_type);

        Self { _pin: pin }
    }
}

impl<'d> Analog<'d> {
    /// Create a GPIO analog function driver for a pin.
    #[inline]
    pub fn new(pin: impl Peripheral<P = impl Pin> + 'd) -> Self {
        let pin = Flex::new(pin);

        pin.set_as_analog();

        Self { _pin: pin }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

mod v2 {
    use super::{Flex, Input, Output, PinLevel};
    use core::convert::Infallible;

    impl<'d> embedded_hal_027::digital::v2::InputPin for Input<'d> {
        type Error = Infallible;
        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Low)
        }

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Hight)
        }
    }

    impl<'d> embedded_hal_027::digital::v2::OutputPin for Output<'d> {
        type Error = Infallible;

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.write(PinLevel::Low);
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.write(PinLevel::Hight);
            Ok(())
        }
    }

    impl<'d> embedded_hal_027::digital::v2::StatefulOutputPin for Output<'d> {
        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Low)
        }

        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Hight)
        }
    }

    impl<'d> embedded_hal_027::digital::v2::ToggleableOutputPin for Output<'d> {
        type Error = Infallible;
        fn toggle(&mut self) -> Result<(), Self::Error> {
            self.write(!(self.read()));
            Ok(())
        }
    }

    impl<'d> embedded_hal_027::digital::v2::InputPin for Flex<'d> {
        type Error = Infallible;
        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Low)
        }

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Hight)
        }
    }

    impl<'d> embedded_hal_027::digital::v2::OutputPin for Flex<'d> {
        type Error = Infallible;

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.write(PinLevel::Low);
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.write(PinLevel::Hight);
            Ok(())
        }
    }

    impl<'d> embedded_hal_027::digital::v2::StatefulOutputPin for Flex<'d> {
        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Low)
        }

        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Hight)
        }
    }

    impl<'d> embedded_hal_027::digital::v2::ToggleableOutputPin for Flex<'d> {
        type Error = Infallible;
        fn toggle(&mut self) -> Result<(), Self::Error> {
            self.write(!(self.read()));
            Ok(())
        }
    }
}

/// 所有 gpio 引脚定义和功能复用定义的宏
macro_rules! gpio_pin_def {
    (
        $gpio_mod: ident, $gpio_port: ident
            $(
                $port_pin_name:ident => $pin_name:ident : $pin_index: expr,
                    [
                        $($Af: ident = $number: expr),*
                    ]
            ),*
    ) => {
        pub mod $gpio_mod {
            use super::*;
            use crate::mcu::peripherals;

            // pub struct $gpio_port;

            pub struct Port {
                pin: u8,
            }

            impl Port {
                pub fn erase_port(self) -> AnyPin {
                    unsafe { AnyPin::steal(GpioPort::$gpio_port, self.pin) }
                }
            }

            pub struct Pins {
                $(
                    pub $port_pin_name: $port_pin_name,
                )*
            }

            impl peripherals::$gpio_port {
                pub fn split(self) -> Pins {
                    // 开启电平
                    self.enable();
                    Pins {
                        $(
                            $port_pin_name: $port_pin_name,
                        )*
                    }
                }

                pub fn enable(&self) {
                    PeripheralClockIndex::$gpio_port.open();
                }

                pub fn reset(&self) {
                    PeripheralClockIndex::$gpio_port.reset();
                }
            }

            $(
                pub struct $port_pin_name;

                impl_peripheral!($port_pin_name);

                impl $port_pin_name {
                    pub fn erase_pin(self) -> Port {
                        Port {
                            pin: $pin_index,
                        }
                    }
                }

                #[allow(clippy::upper_case_acronyms)]
                pub enum $pin_name{
                    $(
                        $Af = $number,
                    )*
                }

                impl From<$pin_name> for PinAF{
                    fn from(value: $pin_name) -> Self {
                        (value as u32).into()
                    }
                }

                impl sealed::Pin for $port_pin_name {
                    fn port_pin(&self) -> u8 {
                        (GpioPort::$gpio_port as u8) << 4 | $pin_index
                    }
                }

                impl Pin for $port_pin_name {
                    fn degrade(self) -> AnyPin {
                        AnyPin {
                            port_pin: (GpioPort::$gpio_port as u8) << 4 | $pin_index
                        }
                    }
                }

                impl From<$port_pin_name> for AnyPin {
                    fn from(value: $port_pin_name) -> Self {
                        // unsafe { AnyPin::steal(GpioPort::$gpio_port, $pin_index) }
                        value.degrade()
                    }
                }
            )*
        }
    };
}

// GPIOA
gpio_pin_def!(gpioa, GPIOA
    PA0 => A0:0,
    [
        SPI2_SCK = 0, USART1_CTS = 1, LED_DATA_B = 3,USART2_CTS = 4,COMP1_OUT = 7,USART2_TX = 9,
        SPI1_MISO = 10,TIM1_CH3 = 13,TIM1_CH1N = 14,IR_OUT = 15
    ],
    PA1 => A1:1,
    [
        SPI1_SCK = 0, USART1_RTS = 1, LED_DATA_C = 3,USART2_RTS = 4, EVENTOUT = 7,USART2_RX = 9,
        SPI1_MOSI = 10,TIM1_CH4 = 13,TIM1_CH2N = 14, MCO = 15
    ],
    PA2 => A2:2,
    [
        SPI1_MOSI = 0, USART1_TX = 1, LED_DATA_D = 3,USART2_TX = 4, COMP2_OUT = 7,SPI1_SCK = 10,
        I2C_SDA = 12, TIM3_CH1 = 13
    ],
    PA3 => A3:3,
    [
        SPI1_MOSI = 0, USART1_TX = 1, LED_DATA_D = 3,USART2_TX = 4, COMP2_OUT = 7,SPI1_SCK = 10,
        I2C_SDA = 12,TIM3_CH1 = 13
    ],
    PA4 => A4:4,
    [
        SPI1_NSS = 0, USART1_CK = 1, SPI2_MOSI = 2,LED_DATA_F = 3,TIM14_CH1 = 4, USART2_CK = 5,
        EVENTOUT = 7,USART2_TX = 9, TIM3_CH3 = 12,RTC_OUT = 13
    ],
    PA5 => A5:5,
    [
        SPI1_SCK = 0, LED_DATA_G = 3, LPTIM1_ETR = 5,EVENTOUT = 7,USART2_RX = 9, TIM3_CH2 = 13,
        MCO = 15
    ],
    PA6 => A6:6,
    [
        SPI1_MISO = 0, TIM3_CH1 = 1, TIM1_BKIN = 2,LED_DATA_DP = 3,TIM16_CH1 = 5, COMP1_OUT = 7,
        USART1_CK = 9, RTC_OUT = 15
    ],
    PA7 => A7:7,
    [
        SPI1_MOSI = 0, TIM3_CH2 = 1, TIM1_CH1N = 2,TIM14_CH1 = 4,TIM17_CH1 = 5, EVENTOUT = 6,
        COMP2_OUT = 7, USART1_TX = 8, USART2_TX = 9, SPI1_MISO = 10, I2C_SDA = 12
    ],
    PA8 => A8:8,
    [
        SPI2_NSS = 0, USART1_CK = 1, TIM1_CH1 = 2,USART2_CK = 4,MCO = 5, EVENTOUT = 7,
        USART1_RX = 8, USART2_RX = 9, SPI1_MOSI = 10, I2C_SCL = 12
    ],
    PA9 => A9:9,
    [
        SPI2_MISO = 0, USART1_TX = 1, TIM1_CH2 = 2,USART2_TX = 4,MCO = 5, I2C_SCL = 6,
        EVENTOUT = 7, USART1_RX = 8, SPI1_SCK = 10, I2C_SDA = 12, TIM1_BKIN = 13
    ],
    PA10 => A10:10,
    [
        SPI2_MOSI = 0, USART1_RX = 1, TIM1_CH3 = 2, USART2_RX = 4, TIM17_BKIN = 5, I2C_SDA = 6,
        EVENTOUT = 7, USART1_TX = 8, SPI1_NSS = 10, I2C_SCL = 12
    ],
    PA11 => A11:11,
    [
        SPI1_MISO = 0, USART1_CTS = 1, TIM1_CH4 = 2, USART2_CTS = 4, EVENTOUT = 5, I2C_SCL = 6,
        COMP1_OUT = 7
    ],
    PA12 => A12:12,
    [
        SPI1_MOSI = 0, USART1_RTS = 1, TIM1_ETR = 2, USART2_RTS = 4, EVENTOUT = 5, I2C_SDA = 6,
        COMP2_OUT = 7
    ],
    PA13 => A13:13,
    [
        SWDIO = 0, IR_OUT = 1, COMP2_OUT = 7, USART1_RX = 8, SPI1_MISO = 10, TIM1_CH2 = 13,
        MCO = 15
    ],
    PA14 => A14:14,
    [
        SWCLK = 0, USART1_TX = 1, USART2_TX = 4, EVENTOUT = 7, MCO = 15
    ],
    PA15 => A15:15,
    [
        SPI1_NSS = 0, USART1_RX = 1, USART2_RX = 4, EVENTOUT = 7, LED_COM0 = 6
    ]
);

// GPIOB
gpio_pin_def!(gpiob, GPIOB
    PB0 => B0:0,[SPI1_NSS = 0, TIM3_CH3 = 1, TIM1_CH2N = 2, EVENTOUT = 5,COMP1_OUT = 7],
    PB1 => B1:1,[TIM14_CH1 = 0, TIM3_CH4 = 1, TIM1_CH3N = 2, EVENTOUT = 7],
    PB2 => B2:2,[USART1_RX = 0, SPI2_SCK = 1, USART2_RX = 3],
    PB3 => B3:3,[SPI1_SCK = 0, TIM1_CH2 = 1, USART1_RTS = 3, USART2_RTS = 4,LED_COM1 = 6, EVENTOUT = 7],
    PB4 => B4:4,[SPI1_MISO = 0, TIM3_CH1 = 1, USART1_CTS = 3, USART2_CTS = 4,TIM17_BKIN = 5, LED_COM2 = 6,  EVENTOUT = 7],
    PB5 => B5:5,[SPI1_MOSI = 0, TIM3_CH2 = 1, TIM16_BKIN = 2, USART1_CK = 3,USART2_CK = 4, LPTIM_IN1 = 5,  LED_COM3 = 6, COMP1_OUT = 7],
    PB6 => B6:6,[USART1_TX = 0, TIM1_CH3 = 1, TIM16_CH1N = 2, SPI2_MISO = 3,USART2_TX = 4, LPTIM_ETR = 5,  I2C_SCL = 6, EVENTOUT = 7],
    PB7 => B7:7,[USART1_RX = 0, SPI2_MOSI = 1, TIM17_CH1N = 2, USART2_RX = 4,  I2C_SDA = 6, EVENTOUT = 7],
    PB8 => B8:8,[SPI2_SCK = 1, TIM16_CH1 = 2, LED_DATA_A = 3, USART2_TX = 4,  I2C_SCL = 6,  EVENTOUT = 7, USART1_TX = 8,
        SPI2_NSS = 11, I2C_SDA = 12, TIM17_CH1 = 13,  IR_OUT = 15]
);

// GPIOF
gpio_pin_def!(gpiof, GPIOF
    PF0_OSC_IN => F0:0,[TIM14_CH1 = 2, SPI2_SCK = 3, USART2_RX = 4, USART1_RX = 8, USART2_TX = 9, I2C_SDA = 12],
    PF1_OSC_OUT => F1:1,[SPI2_MISO = 3, USART2_TX = 4, USART1_TX = 8, USART2_RX = 9, SPI1_NSS = 10, I2C_SCL = 12, TIM14_CH1 = 13],
    PF2_NRST => F2:2,[SPI2_MOSI = 0, USART2_RX = 1, MCO = 3],
    PF3 => F3:3,[USART1_TX = 0, SPI2_MISO = 3, USART2_TX = 4],
    PF4_BOOT0 => F4:4,[]
);
