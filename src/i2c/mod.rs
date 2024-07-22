mod hal;
mod pins;

use core::marker::PhantomData;

use crate::clock::peripheral;
use crate::gpio::{self, AnyPin};
use crate::macro_def::pin_af_for_instance_def;
use crate::mode::Mode;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};

pub struct Master<'d, T: I2cInstance> {
    _t: PhantomData<&'d T>,
}
pub struct Slave<'d, T: I2cInstance> {
    _t: PhantomData<&'d T>,
}

pub trait I2cInstance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

#[derive(PartialEq)]
pub enum I2c {
    I2c1,
}
impl I2c {
    fn enable(&self, en: bool) {
        match *self {
            Self::I2c1 => peripheral::PeripheralClock::I2C.enable(en),
        }
    }

    fn reset(&self) {
        match *self {
            Self::I2c1 => peripheral::PeripheralClock::I2C.reset(),
        }
    }
}

// /// 标准模式下最大时钟频率
// const SPEED_HZ_STAND_CLK_MAX: usize = 2000_000;
// /// 快速模式下最大时钟频率
// const SPEED_HZ_FAST_CLK_MAX: usize = 4000_000;

pub const SPEED_HZ_STAND: usize = 1000_000;
pub const SPEED_HZ_FAST: usize = 4000_000;

// 主从模式
pub enum Rule {
    Master,
    Slave,
}

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

pin_af_for_instance_def!(SdaPin, I2cInstance);
pin_af_for_instance_def!(SclPin, I2cInstance);

macro_rules! impl_sealed_i2c {
    (
        $peripheral: ident, $i2c_id: ident
    ) => {
        impl hal::sealed::Instance for crate::mcu::peripherals::$peripheral {
            fn i2c() -> I2c {
                I2c::$i2c_id
            }
        }
        impl I2cInstance for crate::mcu::peripherals::$peripheral {}
    };
}

impl_sealed_i2c!(I2C, I2c1);

pub struct AnyI2c<'d, T: I2cInstance, M: Mode> {
    _t: PhantomData<&'d T>,
    _mode: PhantomData<M>,
    sda: PeripheralRef<'d, AnyPin>,
    scl: PeripheralRef<'d, AnyPin>,
}

impl<'d, T: I2cInstance, M: Mode> AnyI2c<'d, T, M> {
    fn new_inner(config: Config) -> Result<(), Error> {
        T::i2c().enable(true);
        T::config(config)?;
        Ok(())
    }

    pub fn new_master() -> Master<'d, T> {
        todo!()
    }

    pub fn new_slave() -> Slave<'d, T> {
        todo!()
    }

    pub fn new(
        _i2c: impl Peripheral<P = T>,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        config: Config,
    ) -> Self {
        into_ref!(_i2c, scl, sda);

        // od + pullup
        scl.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::OpenDrain);
        sda.set_instance_af(gpio::PinSpeed::VeryHigh, gpio::PinIoType::OpenDrain);

        let _ = Self::new_inner(config);

        Self {
            _t: PhantomData,
            _mode: PhantomData,
            sda: sda.map_into(),
            scl: scl.map_into(),
        }
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
