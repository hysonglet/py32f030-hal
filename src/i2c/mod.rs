mod future;
mod hal;
pub mod master;
mod pins;
pub mod slave;

use crate::clock::peripheral::{PeripheralClockIndex, PeripheralEnable, PeripheralInterrupt};
use crate::gpio::{self, AnyPin};
use crate::macro_def::{impl_sealed_peripheral_id, pin_af_for_instance_def};
use crate::mode::Mode;
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use enumset::EnumSetType;
pub use master::Master;
pub use slave::Slave;

pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

///  mcu i2c 的索引
#[derive(PartialEq)]
pub enum Id {
    I2c1,
}

impl PeripheralEnable for Id {
    fn clock(&self, en: bool) {
        match *self {
            Self::I2c1 => PeripheralClockIndex::I2C.clock(en),
        }
    }

    fn is_open(&self) -> bool {
        match *self {
            Self::I2c1 => PeripheralClockIndex::I2C.is_open(),
        }
    }

    fn reset(&self) {
        match *self {
            Self::I2c1 => PeripheralClockIndex::I2C.reset(),
        }
    }
}

impl PeripheralInterrupt for Id {
    fn interrupt(&self) -> crate::pac::interrupt {
        match *self {
            Self::I2c1 => crate::pac::interrupt::I2C1,
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

impl_sealed_peripheral_id!(I2C, I2c1);

pub struct AnyI2c<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _mode: PhantomData<M>,
    _sda: PeripheralRef<'d, AnyPin>,
    _scl: PeripheralRef<'d, AnyPin>,
}

impl<'d, T: Instance, M: Mode> AnyI2c<'d, T, M> {
    fn new_inner(config: Config) -> Result<(), Error> {
        T::id().open();
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

#[derive(EnumSetType)]
pub enum Event {
    /// 起始位已发送(Master)                        // 开启控制bit: ITEVTEN
    SB,
    /// 地址已发送(Master) 或 地址匹配(Slave)
    ADD,
    /// 已收到停止(Slave)
    STOPF,
    /// 数据字节传输完成
    BTF,

    /// 接收缓冲区非空                              // 开启控制 bit: ITEVTEN 和ITBUFEN
    RXNE,
    /// 发送缓冲区空
    TXE,

    /// 总线错误                                   // 开启控制 bit: ITERREN
    BERR,
    /// 仲裁丢失(Master)
    ARLO,
    /// 响应失败
    AF,
    /// 过载/欠载
    OVR,
    /// PEC错误
    PECERR,
}

impl Event {
    const fn count() -> usize {
        11
    }
}
