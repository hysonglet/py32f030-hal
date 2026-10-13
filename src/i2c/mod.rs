#[cfg(feature = "embassy")]
mod future;
mod instance;
pub mod master;
mod pins;
pub mod slave;

use crate::clock::peripheral::{
    PeripheralClockIndex, PeripheralIdToClockIndex, PeripheralInterrupt,
};
use crate::gpio::{self, AnyPin};
use crate::macro_def::{impl_sealed_peripheral_id, pin_af_for_instance_def};
use crate::mode::Mode;
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral, PeripheralRef};
use enumset::EnumSetType;
pub use master::Master;
pub use slave::Slave;

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + instance::sealed::Instance + 'static + Send {}

/// MCU I2C index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Id {
    I2c1,
}

impl PeripheralIdToClockIndex for Id {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::I2c1 => PeripheralClockIndex::I2C,
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

/// I2C standard-mode maximum speed: 100K.
pub const SPEED_HZ_STAND: usize = 100_000;
/// I2C fast-mode maximum speed: 400K.
pub const SPEED_HZ_FAST: usize = 400_000;

/// Master / slave mode.
pub enum Rule {
    Master,
    Slave,
}

/// Error type for I2C configuration and operation.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    Busy,
    // PClockToLow,
    PClock,
    SpeedMode,
    Start,
    Address,
    /// Acknowledge failure (AF).
    Acknowledge,
    /// Stop condition.
    Stop,
    Tx,
    RX,
    /// Wait timeout.
    Timeout,
    /// Arbitration lost (ARLO).
    ArbitrationLost,
    /// Bus error (BERR).
    Bus,
    /// Overrun / underrun (OVR).
    Overrun,
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
        T::id().clock().open();
        T::config(config)?;
        Ok(())
    }

    pub fn as_master(self) -> Master<'d, T, M> {
        Master::<'_, T, M>::new()
    }

    /// Convert into slave mode (the own address from the config is written automatically).
    pub fn as_slave(self, address: u8) -> Slave<'d, T, M> {
        T::address(address & 0x7f);
        Slave::<'_, T, M>::new()
    }

    pub fn new(
        _i2c: impl Peripheral<P = T>,
        scl: impl Peripheral<P = impl SclPin<T>> + 'd,
        sda: impl Peripheral<P = impl SdaPin<T>> + 'd,
        config: Config,
    ) -> Result<Self, Error> {
        into_ref!(_i2c, scl, sda);

        // od + pullup
        // Configure the pins to the alternate function, open-drain mode.
        scl.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::OpenDrain);
        sda.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::OpenDrain);

        // Initialize.
        Self::new_inner(config)?;

        Ok(Self {
            _t: PhantomData,
            _mode: PhantomData,
            _sda: sda.map_into(),
            _scl: scl.map_into(),
        })
    }
}

/// I2C peripheral configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// Communication speed (Hz), up to 400K.
    speed: usize,
    /// Own address in slave mode (7-bit).
    own_address: u8,
    /// Enable general call (ENGC).
    general_call: bool,
    /// Disable clock stretching (NOSTRETCH).
    no_stretch: bool,
    /// Enable PEC check.
    pec: bool,
}

impl Default for Config {
    fn default() -> Self {
        // Default speed 100K.
        Self {
            speed: 100_000,
            own_address: 0,
            general_call: false,
            no_stretch: false,
            pec: false,
        }
    }
}

impl Config {
    /// Set the communication speed.
    pub fn speed(self, speed: usize) -> Self {
        Self { speed, ..self }
    }

    /// Set the slave own address (7-bit).
    pub fn own_address(self, address: u8) -> Self {
        Self {
            own_address: address & 0x7f,
            ..self
        }
    }

    /// Enable general call.
    pub fn general_call(self, en: bool) -> Self {
        Self {
            general_call: en,
            ..self
        }
    }

    /// Disable clock stretching.
    pub fn no_stretch(self, en: bool) -> Self {
        Self {
            no_stretch: en,
            ..self
        }
    }

    /// Enable PEC check.
    pub fn pec(self, en: bool) -> Self {
        Self { pec: en, ..self }
    }
}

#[derive(EnumSetType)]
pub enum Event {
    /// Start bit sent (Master).  // enable bit: ITEVTEN
    SB,
    /// Address sent (Master) or address matched (Slave).
    ADD,
    /// Stop received (Slave).
    STOPF,
    /// Data byte transfer finished (BTF).
    BTF,

    /// Receive buffer not empty (RXNE).  // enable bits: ITEVTEN and ITBUFEN
    RXNE,
    /// Transmit buffer empty (TXE).
    TXE,

    /// Bus error (BERR).  // enable bit: ITERREN
    BERR,
    /// Arbitration lost (Master).
    ARLO,
    /// Acknowledge failure (AF).
    AF,
    /// Overrun / underrun (OVR).
    OVR,
    /// PEC error (PECERR).
    PECERR,
}

// impl Event {
//     const fn count() -> usize {
//         11
//     }
// }
