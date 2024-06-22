// pub use embedded_hal::digital::v2::{OutputPin, InputPin};

#[derive(Clone, Copy)]
pub struct Input<MODE = Floating> {
    _mode: PhantomData<MODE>,
}

pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

pub struct Af<MODE> {
    _mode: PhantomData<MODE>,
}

pub struct Analog;
pub struct Floating;
pub struct PullUp;
pub struct PullDown;
pub struct OpenDrain;

pub trait Config {
    fn config<PORT: GpioHal>(pin: Pin);
}

impl<MODE: Config> Config for Input<MODE> {
    fn config<PORT: GpioHal>(pin: Pin) {
        PORT::mode(pin, PinMode::Input);
        MODE::config::<PORT>(pin);
    }
}

impl<MODE: Config> Config for Output<MODE> {
    fn config<PORT: GpioHal>(pin: Pin) {
        PORT::mode(pin, PinMode::Output);
        MODE::config::<PORT>(pin);
    }
}

impl<MODE: Config> Config for Af<MODE> {
    fn config<PORT: GpioHal>(pin: Pin) {
        PORT::mode(pin, PinMode::Af);
        MODE::config::<PORT>(pin);
    }
}

impl Config for PullUp {
    #[inline]
    fn config<PORT: GpioHal>(pin: Pin) {
        PORT::push_pull(pin, PinPullUpDown::PullUp);
    }
}

impl Config for PullDown {
    #[inline]
    fn config<PORT: GpioHal>(pin: Pin) {
        PORT::io_type(pin, PinIoType::PullDown)
    }
}

impl Config for OpenDrain {
    #[inline]
    fn config<PORT: GpioHal>(pin: Pin) {
        PORT::io_type(pin, PinIoType::OpenDrain)
    }
}

impl Config for Analog {
    #[inline]
    fn config<PORT: GpioHal>(pin: Pin) {
        PORT::mode(pin, PinMode::Analog);
    }
}

impl Config for Floating {
    #[inline]
    fn config<PORT: GpioHal>(pin: Pin) {
        PORT::io_type(pin, PinIoType::Floating)
    }
}

pub struct GpioPin<PORT: GpioHal, const PIN: Pin, MODE> {
    _port: PhantomData<PORT>,
    _mode: PhantomData<MODE>,
}

impl<PORT: GpioHal, const PIN: Pin, MODE: Config> GpioPin<PORT, PIN, MODE> {
    pub fn into_input(speed: PinSpeed) -> GpioPin<PORT, PIN, Input<MODE>> {
        PORT::enable(true);
        Input::<MODE>::config::<PORT>(PIN);
        PORT::speed(PIN, speed);

        GpioPin {
            _mode: PhantomData,
            _port: PhantomData,
        }
    }

    pub fn into_output(speed: PinSpeed) -> GpioPin<PORT, PIN, Input<MODE>> {
        PORT::enable(true);
        Output::<MODE>::config::<PORT>(PIN);
        PORT::speed(PIN, speed);

        GpioPin {
            _mode: PhantomData,
            _port: PhantomData,
        }
    }

    pub fn into_analog() -> Self {
        PORT::enable(true);
        Floating::config::<PORT>(PIN);
        Analog::config::<PORT>(PIN);

        GpioPin {
            _mode: PhantomData,
            _port: PhantomData,
        }
    }

    pub fn into_af() -> GpioPin<PORT, PIN, Af<MODE>> {
        PORT::enable(true);
        Floating::config::<PORT>(PIN);
        Af::<MODE>::config::<PORT>(PIN);

        GpioPin {
            _mode: PhantomData,
            _port: PhantomData,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

use core::marker::PhantomData;

use crate::clock::peripheral::GpioClock;
use crate::common::{BitOption, Peripheral};
use crate::pac;

pub type Pin = usize;

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
#[derive(Clone, Copy)]
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
            _ => panic!("Error"),
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

impl From<u32> for PinLevel {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Low,
            1 => Self::Hight,
            _ => panic!("Error PinLevel"),
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

pub trait GpioHal {
    fn push_pull(pin: Pin, push_pull: PinPullUpDown);
    fn output_type(pin: Pin, output_type: PinOutputType);
    fn io_type(pin: Pin, io_type: PinIoType);
    fn lock(pin: Pin, lock: bool);
    // fn open_drain(pin: Pin, open_drain: )
    fn write(pin: Pin, level: PinLevel);
    fn read(pin: Pin) -> PinLevel;
    fn clear(pin: Pin) {
        Self::write(pin, PinLevel::Low)
    }
    fn set(pin: Pin) {
        Self::write(pin, PinLevel::Hight)
    }
    fn reset(pin: Pin);
    fn speed(pin: Pin, speed: PinSpeed);
    fn af(pin: Pin, af: PinAF);
    fn enable(en: bool);
    fn mode(pin: Pin, mode: PinMode);
}

macro_rules! gpio_pin_def {
    (
        $PortStruct: ident, $PortPacMod: ident, $PortPacStruct: ident, $PortClock: ident,
            $(
                $pin_enum: ident =>
                    [
                        $($Af: ident = $number: expr),*
                    ]
            ),*
    ) => {
        struct $PortStruct;

        $(
            #[allow(clippy::upper_case_acronyms)]
            pub enum $pin_enum{
                $(
                    $Af = $number,
                )*
            }

            impl From<$pin_enum> for PinAF{
                fn from(value: $pin_enum) -> Self {
                    (value as u32).into()
                }
            }
        )*

        impl Peripheral for $PortStruct {
            type Target = &'static pac::$PortPacMod::RegisterBlock;
            fn peripheral() -> Self::Target {
                unsafe { pac::$PortPacStruct::PTR.as_ref().unwrap() }
            }
        }

        impl GpioHal for $PortStruct {
            #[inline]
            fn mode(pin: Pin, mode: PinMode) {
                let peripheral = Self::peripheral();
                peripheral.moder.modify(|r, w| unsafe {
                    w.bits(BitOption::bit_mask_idx_modify::<2>(
                        pin,
                        r.bits(),
                        mode as u32,
                    ))
                })
            }

            #[inline]
            fn output_type(pin: Pin, output_type: PinOutputType) {
                let peripheral = Self::peripheral();
                peripheral.otyper.modify(|r, w| unsafe {
                    w.bits(BitOption::bit_mask_idx_modify::<1>(
                        pin,
                        r.bits(),
                        output_type as u32,
                    ))
                })
            }

            fn io_type(pin: Pin, io_type: PinIoType) {
                let (push_pull, output_type) = match io_type {
                    PinIoType::Pullup => (PinPullUpDown::PullUp, PinOutputType::PushPull),
                    PinIoType::PullDown => (PinPullUpDown::PollDown, PinOutputType::PushPull),
                    PinIoType::Floating => (PinPullUpDown::No, PinOutputType::PushPull),
                    PinIoType::OpenDrain => (PinPullUpDown::No, PinOutputType::OpenDrain),
                };

                Self::push_pull(pin, push_pull);
                Self::output_type(pin, output_type)
            }

            #[inline]
            fn speed(pin: Pin, speed: PinSpeed) {
                Self::peripheral().ospeedr.modify(|r, w| unsafe {
                    w.bits(BitOption::bit_mask_idx_modify::<2>(
                        pin,
                        r.bits(),
                        speed as u32,
                    ))
                })
            }

            #[inline]
            fn push_pull(pin: Pin, push_pull: PinPullUpDown) {
                Self::peripheral().pupdr.modify(|r, w| unsafe {
                    w.bits(BitOption::bit_mask_idx_modify::<2>(
                        pin,
                        r.bits(),
                        push_pull as u32,
                    ))
                })
            }

            #[inline]
            fn read(pin: Pin) -> PinLevel {
                let r = Self::peripheral().idr.read().bits();
                BitOption::bit_mask_idx_get::<1>(pin, r).into()
            }

            #[inline]
            fn write(pin: Pin, level: PinLevel) {
                Self::peripheral().odr.modify(|r, w| unsafe {
                    w.bits(BitOption::bit_mask_idx_modify::<1>(
                        pin,
                        r.bits(),
                        level as u32,
                    ))
                })
            }

            #[inline]
            fn lock(_pin: Pin, _lock: bool) {
                // Self::peripheral().lckr.modify(|r, w| unsafe {
                //     w.bits(BitOption::bit_mask_pin_set::<1>(pin, r.bits(), lock as u32))
                // })
                todo!()
            }

            #[inline]
            fn af(pin: Pin, af: PinAF) {
                if pin < 8 {
                    Self::peripheral().afrl.modify(|r, w| unsafe {
                        w.bits(BitOption::bit_mask_idx_modify::<4>(
                            pin,
                            r.bits(),
                            af as u32,
                        ))
                    })
                } else {
                    Self::peripheral().afrh.modify(|r, w| unsafe {
                        w.bits(BitOption::bit_mask_idx_modify::<4>(
                            pin - 8,
                            r.bits(),
                            af as u32,
                        ))
                    })
                }
            }
            #[inline]
            fn clear(pin: Pin) {
                Self::peripheral().bsrr.write(|w| unsafe {
                    w.bits(BitOption::bit_mask_idx_modify::<1>(pin + 16, 0, 1))
                })
            }
            #[inline]
            fn set(pin: Pin) {
                Self::peripheral()
                    .bsrr
                    .write(|w| unsafe { w.bits(BitOption::bit_mask_idx_modify::<1>(pin, 0, 1)) })
            }
            #[inline]
            fn reset(pin: Pin) {
                Self::peripheral()
                    .brr
                    .write(|w| unsafe { w.bits(1 << pin) })
            }
            #[inline]
            fn enable(en: bool) {
                GpioClock::$PortClock.enable(en)
            }
        }
    };
}

gpio_pin_def!(GpioA, gpioa, GPIOA, GPIOA,
    PA0 => [
        SPI2_SCK = 0, USART1_CTS = 1, LED_DATA_B = 3,USART2_CTS = 4,COMP1_OUT = 7,USART2_TX = 9,
        SPI1_MISO = 10,TIM1_CH3 = 13,TIM1_CH1N = 14,IR_OUT = 15
    ],
    PA1 => [
        SPI1_SCK = 0, USART1_RTS = 1, LED_DATA_C = 3,USART2_RTS = 4, EVENTOUT = 7,USART2_RX = 9,
        SPI1_MOSI = 10,TIM1_CH4 = 13,TIM1_CH2N = 14, MCO = 15
    ],
    PA3 => [
        SPI1_MOSI = 0, USART1_TX = 1, LED_DATA_D = 3,USART2_TX = 4, COMP2_OUT = 7,SPI1_SCK = 10,
        I2C_SDA = 12,TIM3_CH1 = 13
    ],
    PA4 => [
        SPI1_NSS = 0, USART1_CK = 1, SPI2_MOSI = 2,LED_DATA_F = 3,TIM14_CH1 = 4, USART2_CK = 5,
        EVENTOUT = 7,USART2_TX = 9, TIM3_CH3 = 12,RTC_OUT = 13
    ],
    PA5 => [
        SPI1_SCK = 0, LED_DATA_G = 3, LPTIM1_ETR = 5,EVENTOUT = 7,USART2_RX = 9, TIM3_CH2 = 13,
        MCO = 15
    ],
    PA6 => [
        SPI1_MISO = 0, TIM3_CH1 = 1, TIM1_BKIN = 2,LED_DATA_DP = 3,TIM16_CH1 = 5, COMP1_OUT = 7,
        USART1_CK = 9, RTC_OUT = 15
    ],
    PA7 => [
        SPI1_MOSI = 0, TIM3_CH2 = 1, TIM1_CH1N = 2,TIM14_CH1 = 4,TIM17_CH1 = 5, EVENTOUT = 6,
        COMP2_OUT = 7, USART1_TX = 8, USART2_TX = 9, SPI1_MISO = 10, I2C_SDA = 12
    ],
    PA8 => [
        SPI2_NSS = 0, USART1_CK = 1, TIM1_CH1 = 2,USART2_CK = 4,MCO = 5, EVENTOUT = 7,
        USART1_RX = 8, USART2_RX = 9, SPI1_MOSI = 10, I2C_SCL = 12
    ],
    PA9 => [
        SPI2_MISO = 0, USART1_TX = 1, TIM1_CH2 = 2,USART2_TX = 4,MCO = 5, I2C_SCL = 6,
        EVENTOUT = 7, USART1_RX = 8, SPI1_SCK = 10, I2C_SDA = 12, TIM1_BKIN = 13
    ],
    PA10 => [
        SPI2_MOSI = 0, USART1_RX = 1, TIM1_CH3 = 2, USART2_RX = 4, TIM17_BKIN = 5, I2C_SDA = 6,
        EVENTOUT = 7, USART1_TX = 8, SPI1_NSS = 10, I2C_SCL = 12
    ],
    PA11 => [
        SPI1_MISO = 0, USART1_CTS = 1, TIM1_CH4 = 2, USART2_CTS = 4, EVENTOUT = 5, I2C_SCL = 6,
        COMP1_OUT = 7
    ],
    PA12 => [
        SPI1_MOSI = 0, USART1_RTS = 1, TIM1_ETR = 2, USART2_RTS = 4, EVENTOUT = 5, I2C_SDA = 6,
        COMP2_OUT = 7
    ],
    PA13 => [
        SWDIO = 0, IR_OUT = 1, COMP2_OUT = 7, USART1_RX = 8, SPI1_MISO = 10, TIM1_CH2 = 13,
        MCO = 15
    ],
    PA14 => [
        SWCLK = 0, USART1_TX = 1, USART2_TX = 4, EVENTOUT = 7, MCO = 15
    ],
    PA15 => [
        SPI1_NSS = 0, USART1_RX = 1, USART2_RX = 4, EVENTOUT = 7, LED_COM0 = 6
    ]
);

gpio_pin_def!(GpioB, gpiob, GPIOB, GPIOB,
    PB0 => [SPI1_NSS = 0, TIM3_CH3 = 1, TIM1_CH2N = 2, EVENTOUT = 5,COMP1_OUT = 7],
    PB1 => [TIM14_CH1 = 0, TIM3_CH4 = 1, TIM1_CH3N = 2, EVENTOUT = 7],
    PB2 => [USART1_RX = 0, SPI2_SCK = 1, USART2_RX = 3],
    PB3 => [SPI1_SCK = 0, TIM1_CH2 = 1, USART1_RTS = 3, USART2_RTS = 4,LED_COM1 = 6, EVENTOUT = 7],
    PB4 => [SPI1_MISO = 0, TIM3_CH1 = 1, USART1_CTS = 3, USART2_CTS = 4,TIM17_BKIN = 5, LED_COM2 = 6,  EVENTOUT = 7],
    PB5 => [SPI1_MOSI = 0, TIM3_CH2 = 1, TIM16_BKIN = 2, USART1_CK = 3,USART2_CK = 4, LPTIM_IN1 = 5,  LED_COM3 = 6, COMP1_OUT = 7],
    PB6 => [USART1_TX = 0, TIM1_CH3 = 1, TIM16_CH1N = 2, SPI2_MISO = 3,USART2_TX = 4, LPTIM_ETR = 5,  I2C_SCL = 6, EVENTOUT = 7],
    PB7 => [USART1_RX = 0, SPI2_MOSI = 1, TIM17_CH1N = 2, USART2_RX = 4,  I2C_SDA = 6, EVENTOUT = 7],
    PB8 => [SPI2_SCK = 1, TIM16_CH1 = 2, LED_DATA_A = 3, USART2_TX = 4,  I2C_SCL = 6,  EVENTOUT = 7, USART1_TX = 8,
        SPI2_NSS = 11, I2C_SDA = 12, TIM17_CH1 = 13,  IR_OUT = 15]
);

gpio_pin_def!(GpioF, gpiof, GPIOF, GPIOF,
    PF0_OSC_IN => [TIM14_CH1 = 2, SPI2_SCK = 3, USART2_RX = 4, USART1_RX = 8, USART2_TX = 9, I2C_SDA = 12],
    PF1_OSC_OUT => [SPI2_MISO = 3, USART2_TX = 4, USART1_TX = 8, USART2_RX = 9, SPI1_NSS = 10, I2C_SCL = 12, TIM14_CH1 = 13],
    PF2_NRST => [SPI2_MOSI = 0, USART2_RX = 1, MCO = 3],
    PF3 => [USART1_TX = 0, SPI2_MISO = 3, USART2_TX = 4],
    PF4_BOOT0 => []
);

// struct GpioA;

// impl Peripheral for GpioA {
//     type Target = &'static pac::gpioa::RegisterBlock;
//     fn peripheral() -> Self::Target {
//         unsafe { pac::GPIOA::PTR.as_ref().unwrap() }
//     }
// }

// impl GpioHal for GpioA {
//     #[inline]
//     fn mode(pin: Pin, mode: PinMode) {
//         let peripheral = Self::peripheral();
//         peripheral.moder.modify(|r, w| unsafe {
//             w.bits(BitOption::bit_mask_idx_modify::<2>(
//                 pin,
//                 r.bits(),
//                 mode as u32,
//             ))
//         })
//     }

//     #[inline]
//     fn output_type(pin: Pin, output_type: PinOutputType) {
//         let peripheral = Self::peripheral();
//         peripheral.otyper.modify(|r, w| unsafe {
//             w.bits(BitOption::bit_mask_idx_modify::<1>(
//                 pin,
//                 r.bits(),
//                 output_type as u32,
//             ))
//         })
//     }

//     fn io_type(pin: Pin, io_type: PinIoType) {
//         let (push_pull, output_type) = match io_type {
//             PinIoType::Pullup => (PinPullUpDown::PullUp, PinOutputType::PushPull),
//             PinIoType::PullDown => (PinPullUpDown::PollDown, PinOutputType::PushPull),
//             PinIoType::Floating => (PinPullUpDown::No, PinOutputType::PushPull),
//             PinIoType::OpenDrain => (PinPullUpDown::No, PinOutputType::OpenDrain),
//         };

//         Self::push_pull(pin, push_pull);
//         Self::output_type(pin, output_type)
//     }

//     #[inline]
//     fn speed(pin: Pin, speed: PinSpeed) {
//         Self::peripheral().ospeedr.modify(|r, w| unsafe {
//             w.bits(BitOption::bit_mask_idx_modify::<2>(
//                 pin,
//                 r.bits(),
//                 speed as u32,
//             ))
//         })
//     }

//     #[inline]
//     fn push_pull(pin: Pin, push_pull: PinPullUpDown) {
//         Self::peripheral().pupdr.modify(|r, w| unsafe {
//             w.bits(BitOption::bit_mask_idx_modify::<2>(
//                 pin,
//                 r.bits(),
//                 push_pull as u32,
//             ))
//         })
//     }

//     #[inline]
//     fn read(pin: Pin) -> PinLevel {
//         let r = Self::peripheral().idr.read().bits();
//         BitOption::bit_mask_idx_get::<1>(pin, r).into()
//     }

//     #[inline]
//     fn write(pin: Pin, level: PinLevel) {
//         Self::peripheral().odr.modify(|r, w| unsafe {
//             w.bits(BitOption::bit_mask_idx_modify::<1>(
//                 pin,
//                 r.bits(),
//                 level as u32,
//             ))
//         })
//     }

//     #[inline]
//     fn lock(_pin: Pin, _lock: bool) {
//         // Self::peripheral().lckr.modify(|r, w| unsafe {
//         //     w.bits(BitOption::bit_mask_pin_set::<1>(pin, r.bits(), lock as u32))
//         // })
//         todo!()
//     }

//     #[inline]
//     fn af(pin: Pin, af: PinAF) {
//         if pin < 8 {
//             Self::peripheral().afrl.modify(|r, w| unsafe {
//                 w.bits(BitOption::bit_mask_idx_modify::<4>(
//                     pin,
//                     r.bits(),
//                     af as u32,
//                 ))
//             })
//         } else {
//             Self::peripheral().afrh.modify(|r, w| unsafe {
//                 w.bits(BitOption::bit_mask_idx_modify::<4>(
//                     pin - 8,
//                     r.bits(),
//                     af as u32,
//                 ))
//             })
//         }
//     }
//     #[inline]
//     fn clear(pin: Pin) {
//         Self::peripheral()
//             .bsrr
//             .write(|w| unsafe { w.bits(BitOption::bit_mask_idx_modify::<1>(pin + 16, 0, 1)) })
//     }
//     #[inline]
//     fn set(pin: Pin) {
//         Self::peripheral()
//             .bsrr
//             .write(|w| unsafe { w.bits(BitOption::bit_mask_idx_modify::<1>(pin, 0, 1)) })
//     }
//     #[inline]
//     fn reset(pin: Pin) {
//         Self::peripheral()
//             .brr
//             .write(|w| unsafe { w.bits(1 << pin) })
//     }
//     #[inline]
//     fn enable(en: bool) {
//         GpioClock::GPIOA.enable(en)
//     }
// }

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq)]
pub enum GpioPort {
    GpioA,
    GpioB,
    GpioF,
}

// #[derive(Clone, Copy, PartialEq)]
// pub enum PinNumber {
//     Pin0 = 0,
//     Pin1 = 1,
//     Pin2 = 2,
//     Pin3 = 3,
//     Pin4 = 4,
//     Pin5 = 5,
//     Pin6 = 6,
//     Pin7 = 7,
//     Pin8 = 8,
//     Pin9 = 9,
//     Pin10 = 10,
//     Pin11 = 11,
//     Pin12 = 12,
//     Pin13 = 13,
//     Pin14 = 14,
//     Pin15 = 15,
// }

impl<MODE: Config> AnyPin<MODE> {
    fn config(&self, speed: PinSpeed) {
        match self.port {
            GpioPort::GpioA => {
                GpioA::enable(true);
                MODE::config::<GpioA>(self.pin);
                GpioA::speed(self.pin, speed);
            }
            GpioPort::GpioB => {}
            GpioPort::GpioF => {}
        }
    }

    fn write(&self, level: PinLevel) {
        match self.port {
            GpioPort::GpioA => GpioA::write(self.pin, level),
            GpioPort::GpioB => {}
            GpioPort::GpioF => {}
        }
    }

    fn read(&self) -> PinLevel {
        match self.port {
            GpioPort::GpioA => GpioA::read(self.pin),
            GpioPort::GpioB => {
                todo!()
            }
            GpioPort::GpioF => {
                todo!()
            }
        }
    }
}

pub struct AnyPin<MODE: Config> {
    port: GpioPort,
    pin: Pin,
    _mode: PhantomData<MODE>,
}

impl<MODE: Config> AnyPin<MODE> {
    /// Unsafely crate a new type-erase pin
    ///
    /// # Safety
    ///
    /// You must ensure that you’re only using one instance of this type at a time.
    pub unsafe fn steal(port: GpioPort, pin: Pin) -> Self {
        assert!(pin < 16);
        Self {
            port,
            pin,
            _mode: PhantomData,
        }
    }
}

impl<MODE: Config> AnyPin<MODE> {
    pub fn into_input(self, speed: PinSpeed) -> AnyPin<Input<MODE>> {
        let any_pin: AnyPin<Input<MODE>> = AnyPin {
            port: self.port,
            pin: self.pin,
            _mode: PhantomData,
        };

        any_pin.config(speed);
        any_pin
    }

    pub fn into_output(self, speed: PinSpeed) -> AnyPin<Output<MODE>> {
        let any_pin: AnyPin<Output<MODE>> = AnyPin {
            port: self.port,
            pin: self.pin,
            _mode: PhantomData,
        };

        any_pin.config(speed);
        any_pin
    }

    pub fn into_analog(self) -> AnyPin<Analog> {
        let any_pin: AnyPin<Analog> = AnyPin {
            port: self.port,
            pin: self.pin,
            _mode: PhantomData,
        };
        any_pin.config(PinSpeed::VeryLow);
        any_pin
    }

    pub fn into_af(self) -> AnyPin<Af<MODE>> {
        let any_pin: AnyPin<Af<MODE>> = AnyPin {
            port: self.port,
            pin: self.pin,
            _mode: PhantomData,
        };
        any_pin.config(PinSpeed::VeryLow);
        any_pin
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
use embedded_hal::digital;

mod v2 {
    use core::convert::Infallible;

    use super::*;
    impl<PORT: GpioHal, const PIN: Pin, MODE: Config> digital::v2::InputPin
        for GpioPin<PORT, PIN, Input<MODE>>
    {
        type Error = Infallible;
        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(PORT::read(PIN) == PinLevel::Low)
        }

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(PORT::read(PIN) == PinLevel::Hight)
        }
    }

    impl From<digital::v2::PinState> for PinLevel {
        fn from(value: digital::v2::PinState) -> Self {
            if value == digital::v2::PinState::High {
                PinLevel::Hight
            } else {
                PinLevel::Low
            }
        }
    }

    impl<PORT: GpioHal, const PIN: Pin, MODE: Config> digital::v2::OutputPin
        for GpioPin<PORT, PIN, Input<MODE>>
    {
        type Error = Infallible;

        fn set_low(&mut self) -> Result<(), Self::Error> {
            PORT::write(PIN, PinLevel::Low);
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            PORT::write(PIN, PinLevel::Hight);
            Ok(())
        }
    }

    impl<MODE: Config> digital::v2::InputPin for AnyPin<Input<MODE>> {
        type Error = Infallible;
        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Low)
        }
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == PinLevel::Hight)
        }
    }

    impl<MODE: Config> digital::v2::OutputPin for AnyPin<Output<MODE>> {
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
}
