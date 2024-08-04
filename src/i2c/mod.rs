mod hal;
pub mod master;
mod pins;
pub mod slave;

use core::marker::PhantomData;

use crate::clock::peripheral::{PeripheralClock, PeripheralEnable};
use crate::gpio::{self, AnyPin};
use crate::macro_def::pin_af_for_instance_def;
use crate::mode::Mode;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
pub use master::Master;
pub use slave::Slave;

pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

///  mcu i2c 的索引
#[derive(PartialEq)]
pub enum I2c {
    I2c1,
}

impl PeripheralEnable for I2c {
    fn enable(&self, en: bool) {
        match *self {
            Self::I2c1 => PeripheralClock::I2C.enable(en),
        }
    }

    fn reset(&self) {
        match *self {
            Self::I2c1 => PeripheralClock::I2C.reset(),
        }
    }
}

/// IIC 标准模式最快速度: 100K
pub const SPEED_HZ_STAND: usize = 100_000;
/// IIC 快速模式最快速度: 400K
pub const SPEED_HZ_FAST: usize = 400_000;

/// 主从模式
pub enum Rule {
    Master,
    Slave,
}

/// IIC 配置和运行的错误类型
#[derive(Debug)]
pub enum Error {
    Busy,
    // PClockToLow,
    PClock,
    SpeedMode,
    Start,
    Address,
    Stop,
    Tx,
    RX,
}

pin_af_for_instance_def!(SdaPin, Instance);
pin_af_for_instance_def!(SclPin, Instance);

macro_rules! impl_sealed_i2c {
    (
        $peripheral: ident, $i2c_id: ident
    ) => {
        impl hal::sealed::Instance for crate::mcu::peripherals::$peripheral {
            fn i2c() -> I2c {
                I2c::$i2c_id
            }
        }
        impl Instance for crate::mcu::peripherals::$peripheral {}
    };
}

impl_sealed_i2c!(I2C, I2c1);

pub struct AnyI2c<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _mode: PhantomData<M>,
    _sda: PeripheralRef<'d, AnyPin>,
    _scl: PeripheralRef<'d, AnyPin>,
}

impl<'d, T: Instance, M: Mode> AnyI2c<'d, T, M> {
    fn new_inner(config: Config) -> Result<(), Error> {
        T::i2c().enable(true);
        T::config(config)?;
        Ok(())
    }

    pub fn as_master(self) -> Master<'d, T, M> {
        Master::<'_, T, M>::new()
    }

    pub fn as_slave() -> Slave<'d, T, M> {
        todo!()
    }

    pub fn new(
        _i2c: impl Peripheral<P = T>,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        into_ref!(_i2c, scl, sda);

        // od + pullup
        // 配置引脚功能为AF功能，开漏模式
        scl.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::OpenDrain);
        sda.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::OpenDrain);

        // 初始化
        Self::new_inner(config)?;

        Ok(Self {
            _t: PhantomData,
            _mode: PhantomData,
            _sda: sda.map_into(),
            _scl: scl.map_into(),
        })
    }
}

/// I2C 外设配置
pub struct Config {
    speed: usize,
}

impl Default for Config {
    fn default() -> Self {
        // 默认速度100K
        Self { speed: 100_000 }
    }
}

impl Config {
    pub fn speed(self, speed: usize) -> Self {
        Self { speed }
    }
}