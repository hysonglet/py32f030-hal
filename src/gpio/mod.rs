mod hal;

use core::ops::Not;
use hal::*;

use crate::clock::peripheral::GpioClock;
use embassy_hal_internal::{impl_peripheral, into_ref, Peripheral, PeripheralRef};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GpioPort {
    GPIOA = 0,
    GPIOB = 1,
    GPIOF = 2,
}

impl GpioPort {
    fn enable(&self, en: bool) {
        match *self {
            GpioPort::GPIOA => GpioClock::GPIOA.enable(en),
            GpioPort::GPIOB => GpioClock::GPIOB.enable(en),
            GpioPort::GPIOF => GpioClock::GPIOF.enable(en),
        }
    }

    fn reset(&self) {
        match *self {
            GpioPort::GPIOA => GpioClock::GPIOA.reset(),
            GpioPort::GPIOB => GpioClock::GPIOB.reset(),
            GpioPort::GPIOF => GpioClock::GPIOF.reset(),
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

// 定义 enum PinMode
#[derive(Clone, Copy)]
pub enum PinMode {
    Input = 0,
    Output = 1,
    Af = 2,
    Analog = 3,
}

// 定义 enum PinSpeed
#[derive(Clone, Copy)]
pub enum PinSpeed {
    VeryLow = 0,
    Low = 1,
    High = 2,
    VeryHigh = 3,
}

// 定义 enum PinAf
#[derive(Debug, Clone, Copy)]
pub enum PinAF {
    AF0 = 0,
    AF1 = 1,
    AF2 = 2,
    AF3 = 3,
    AF4 = 4,
    AF5 = 5,
    AF6 = 6,
    AF7 = 7,
    AF8 = 8,
    AF9 = 9,
    AF10 = 10,
    AF11 = 11,
    AF12 = 12,
    AF13 = 13,
    AF14 = 14,
    AF15 = 15,
}

impl From<u32> for PinAF {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::AF0,
            1 => Self::AF1,
            2 => Self::AF2,
            3 => Self::AF3,
            4 => Self::AF4,
            5 => Self::AF5,
            6 => Self::AF6,
            7 => Self::AF7,
            8 => Self::AF8,
            9 => Self::AF9,
            10 => Self::AF10,
            11 => Self::AF11,
            12 => Self::AF12,
            13 => Self::AF13,
            14 => Self::AF14,
            15 => Self::AF15,
            _ => unreachable!(),
        }
    }
}

// 定义 enum PinPullUpDown
#[derive(Clone, Copy)]
pub enum PinPullUpDown {
    No = 0,
    PullUp = 1,
    PollDown = 2,
}

#[derive(Clone, Copy)]
pub enum PinOutputType {
    PushPull = 0,
    OpenDrain = 1,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PinIoType {
    Floating,
    Pullup,
    PullDown,
    OpenDrain,
}

impl PinIoType {
    fn split(self) -> (PinPullUpDown, PinOutputType) {
        let (push_pull, output_type) = match self {
            PinIoType::Pullup => (PinPullUpDown::PullUp, PinOutputType::PushPull),
            PinIoType::PullDown => (PinPullUpDown::PollDown, PinOutputType::PushPull),
            PinIoType::Floating => (PinPullUpDown::No, PinOutputType::PushPull),
            PinIoType::OpenDrain => (PinPullUpDown::No, PinOutputType::OpenDrain),
        };
        (push_pull, output_type)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PinLock {
    Unlock = 0,
    Lock = 1,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PinLevel {
    Low = 0,
    Hight = 1,
}

impl Not for PinLevel {
    type Output = Self;
    fn not(self) -> Self::Output {
        if self == Self::Low {
            Self::Hight
        } else {
            Self::Low
        }
    }
}

impl From<u32> for PinLevel {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Low,
            1 => Self::Hight,
            _ => unreachable!(),
        }
    }
}

impl From<PinLevel> for bool {
    fn from(value: PinLevel) -> Self {
        PinLevel::Hight == value
    }
}

#[derive(Debug)]
pub enum Error {}

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

pub struct Flex<'d> {
    pub(crate) pin: PeripheralRef<'d, AnyPin>,
}

pub trait Pin: Peripheral<P = Self> + Into<AnyPin> + sealed::Pin + Sized + 'static {
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

    pub fn port(&self) -> GpioPort {
        self.pin.port()
    }

    pub fn pin(&self) -> usize {
        self.pin.pin()
    }

    pub fn set_as_input(&self, pull: PinPullUpDown, speed: PinSpeed) {
        critical_section::with(|_| {
            self.pin.set_mode(PinMode::Input);
            self.pin.set_push_pull(pull);
            self.pin.set_speed(speed);
        });
    }

    pub fn set_as_output(&self, io_type: PinIoType, speed: PinSpeed) {
        critical_section::with(|_| {
            self.pin.set_mode(PinMode::Output);
            self.pin.set_io_type(io_type);
            self.pin.set_speed(speed);
        });
    }

    pub fn set_as_analog(&self) {
        critical_section::with(|_| {
            self.pin.set_mode(PinMode::Analog);
        });
    }

    pub fn set_as_af(&self, af: PinAF, speed: PinSpeed, io_type: PinIoType) {
        critical_section::with(|_| {
            self.pin.set_mode(PinMode::Af);
            self.pin.set_af(af);
            self.pin.set_speed(speed);
            self.pin.set_io_type(io_type);
        });
    }

    pub fn set_push_pull(&self, push_pull: PinPullUpDown) {
        self.pin.set_push_pull(push_pull);
    }

    pub fn set_open_drain(&self, open_drain: PinOutputType) {
        self.pin.set_output_type(open_drain);
    }

    pub fn set_io_type(&self, io_type: PinIoType) {
        self.pin.set_io_type(io_type)
    }

    pub fn read(&self) -> PinLevel {
        self.pin.read()
    }

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

pub struct Input<'d> {
    pub(crate) pin: Flex<'d>,
}
pub struct Output<'d> {
    pub(crate) pin: Flex<'d>,
}
pub struct Af<'d> {
    pub(crate) _pin: Flex<'d>,
}

pub struct Analog<'d> {
    pub(crate) _pin: Flex<'d>,
}

impl<'d> Input<'d> {
    pub fn new(
        pin: impl Peripheral<P = impl Pin> + 'd,
        pull: PinPullUpDown,
        speed: PinSpeed,
    ) -> Self {
        let pin = Flex::new(pin);

        pin.set_as_input(pull, speed);

        Self { pin }
    }

    pub fn read(&self) -> PinLevel {
        self.pin.read()
    }
}

impl<'d> Output<'d> {
    pub fn new(
        pin: impl Peripheral<P = impl Pin> + 'd,
        io_type: PinIoType,
        speed: PinSpeed,
    ) -> Self {
        let pin = Flex::new(pin);

        pin.set_as_output(io_type, speed);

        Self { pin }
    }

    pub fn read(&self) -> PinLevel {
        self.pin.read()
    }

    pub fn write(&self, level: PinLevel) {
        self.pin.write(level)
    }
}

impl<'d> Af<'d> {
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
    pub fn new(pin: impl Peripheral<P = impl Pin> + 'd) -> Self {
        let pin = Flex::new(pin);

        pin.set_as_analog();

        Self { _pin: pin }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod v2 {
    use super::{Flex, Input, Output, PinLevel};
    use core::convert::Infallible;

    impl<'d> embedded_hal::digital::v2::InputPin for Input<'d> {
        type Error = Infallible;
        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Low)
        }

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Hight)
        }
    }

    impl<'d> embedded_hal::digital::v2::OutputPin for Output<'d> {
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

    impl<'d> embedded_hal::digital::v2::StatefulOutputPin for Output<'d> {
        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Low)
        }

        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Hight)
        }
    }

    impl<'d> embedded_hal::digital::v2::ToggleableOutputPin for Output<'d> {
        type Error = Infallible;
        fn toggle(&mut self) -> Result<(), Self::Error> {
            self.write(!(self.read()));
            Ok(())
        }
    }

    impl<'d> embedded_hal::digital::v2::InputPin for Flex<'d> {
        type Error = Infallible;
        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Low)
        }

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Hight)
        }
    }

    impl<'d> embedded_hal::digital::v2::OutputPin for Flex<'d> {
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

    impl<'d> embedded_hal::digital::v2::StatefulOutputPin for Flex<'d> {
        fn is_set_low(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Low)
        }

        fn is_set_high(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Hight)
        }
    }

    impl<'d> embedded_hal::digital::v2::ToggleableOutputPin for Flex<'d> {
        type Error = Infallible;
        fn toggle(&mut self) -> Result<(), Self::Error> {
            self.write(!(self.read()));
            Ok(())
        }
    }
}

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
                    self.enable(true);
                    Pins {
                        $(
                            $port_pin_name: $port_pin_name,
                        )*
                    }
                }

                pub fn enable(&self, en: bool) {
                    GpioPort::$gpio_port.enable(en);
                }

                pub fn reset(&self) {
                    GpioPort::$gpio_port.reset();
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

gpio_pin_def!(gpiof, GPIOF
    PF0_OSC_IN => F0:0,[TIM14_CH1 = 2, SPI2_SCK = 3, USART2_RX = 4, USART1_RX = 8, USART2_TX = 9, I2C_SDA = 12],
    PF1_OSC_OUT => F1:1,[SPI2_MISO = 3, USART2_TX = 4, USART1_TX = 8, USART2_RX = 9, SPI1_NSS = 10, I2C_SCL = 12, TIM14_CH1 = 13],
    PF2_NRST => F2:2,[SPI2_MOSI = 0, USART2_RX = 1, MCO = 3],
    PF3 => F3:3,[USART1_TX = 0, SPI2_MISO = 3, USART2_TX = 4],
    PF4_BOOT0 => F4:4,[]
);
