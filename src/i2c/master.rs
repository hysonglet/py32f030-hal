#[cfg(feature = "embassy")]
use enumset::EnumSet;

#[cfg(feature = "embassy")]
use super::future::EventFuture;
use super::instance::sealed::WAIT_FLAG_TIMEOUT;
use super::{Error, Event, Instance};
use crate::delay::wait_for_true_timeout_block;
#[cfg(feature = "embassy")]
use crate::mode::Async;
use crate::{
    clock::peripheral::PeripheralInterrupt,
    mode::{Blocking, Mode},
};
use core::marker::PhantomData;
use embedded_hal::i2c::Operation;

/// Master role.
pub struct Master<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,
}

impl<'d, T: Instance, M: Mode> Master<'d, T, M> {
    pub(super) fn new() -> Self {
        if M::is_async() {
            T::id().enable_interrupt();
        }
        Self { _t: PhantomData }
    }
}

impl<'d, T: Instance, M: Mode> Drop for Master<'d, T, M> {
    fn drop(&mut self) {
        if M::is_async() {
            T::id().disable_interrupt();
        }
    }
}

impl<'d, T: Instance> Master<'d, T, Blocking> {
    pub fn write_block(&self, address: u8, buf: &[u8]) -> Result<usize, Error> {
        T::clear_pos();

        T::start();
        // SB=1: cleared by reading SR1 then writing the data register.
        wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::SB)).map_err(
            |_| {
                T::event_clear(Event::ARLO);
                Error::Start
            },
        )?;

        T::transmit(address << 1);

        // ADDR=1: cleared by reading SR1 then SR2.
        wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::ADD)).map_err(
            |_| {
                // Self::debug();
                // Clear the AF flag.
                T::event_clear(Event::AF);
                T::stop();
                Error::Address
            },
        )?;
        T::event_clear(Event::ADD);

        // The TRA bit indicates whether the master is in receiver or transmitter mode.

        let mut iter = buf.iter();
        if let Some(d) = iter.next() {
            // EV8_1: TxE=1, shift register empty, data register empty, write Data1 to DR.
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::TXE))
                .map_err(|_| Error::Tx)?;
            T::transmit(*d);
        }

        // Then send the remaining data.
        for t in iter {
            // EV8: TxE=1, shift register not empty, data register empty, write Data2 to DR,
            // this bit is cleared.
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::TXE)).map_err(
                |_| {
                    T::stop();
                    Error::Tx
                },
            )?;
            T::transmit(*t);
        }

        // EV8_2: TxE=1, BTF=1, write the Stop bit; when hardware sends the Stop bit,
        // TxE and BTF are cleared.
        wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::TXE)).map_err(
            |_| {
                T::stop();
                Error::Tx
            },
        )?;

        wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::TXE)).map_err(
            |_| {
                T::stop();
                Error::Tx
            },
        )?;

        wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::BTF)).map_err(
            |_| {
                T::stop();
                Error::Tx
            },
        )?;

        T::stop();

        Ok(buf.len())
    }

    pub fn read_block(&self, address: u8, buf: &mut [u8]) -> Result<usize, Error> {
        T::master_receive_block(address, buf)
    }
}


impl<'d, T: Instance> embedded_hal::i2c::ErrorType for Master<'d, T, Blocking> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_hal::i2c::I2c for Master<'d, T, Blocking> {
    #[inline]
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        for op in operations {
            match op {
                Operation::Write(buf) => {
                    self.write_block(address, buf)?;
                }
                Operation::Read(buf) => {
                    self.read_block(address, buf)?;
                }
            }
        }
        Ok(())
    }
}

impl<'d, T: Instance> embedded_hal_027::blocking::i2c::Write for Master<'d, T, Blocking> {
    type Error = Error;
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write_block(address, bytes)
            .map_or_else(Err, |_| Ok(()))
    }
}

impl<'d, T: Instance> embedded_hal_027::blocking::i2c::Read for Master<'d, T, Blocking> {
    type Error = Error;
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.read_block(address, buffer)
            .map_or_else(Err, |_| Ok(()))
    }
}

impl<'d, T: Instance> embedded_hal_027::blocking::i2c::WriteRead for Master<'d, T, Blocking> {
    type Error = Error;
    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.write_block(address, bytes)
            .map_or_else(Err, |_| Ok(()))?;
        self.read_block(address, buffer)
            .map_or_else(Err, |_| Ok(()))
    }
}

///////

#[cfg(feature = "embassy")]
impl<'d, T: Instance> Master<'d, T, Async> {
    pub async fn read(&self, address: u8, buf: &mut [u8]) -> Result<usize, Error> {
        let block = T::block();

        T::start();

        // EV5: SB=1, read SR1 then write DR to clear this bit.
        EventFuture::<T>::new(EnumSet::empty() | Event::SB).await?;

        T::transmit((address << 1) | 1);

        // EV6: ADDR, read SR1 then SR2 to clear this bit.
        EventFuture::<T>::new(EnumSet::empty() | Event::ADD).await?;

        let len = buf.len();

        let mut enumerate = buf.iter_mut().enumerate();
        while let Some((idx, p)) = enumerate.next() {
            let remain = len - idx;
            if remain > 2 {
                T::ack(true);
                // EV7: RxNE=1, read DR to clear this bit.
                EventFuture::<T>::new(EnumSet::empty() | Event::RXNE).await?;

                // Read the data.
                *p = block.dr.read().dr().bits();

                if T::event_flag(Event::BTF) {
                    T::ack(true);
                    let (_, p) = enumerate.next().unwrap();
                    *p = block.dr.read().dr().bits();
                }
            } else if remain == 2 {
                T::ack(false);
                // EV7: RxNE=1, read DR to clear this bit.
                EventFuture::<T>::new(EnumSet::empty() | Event::RXNE).await?;
                // Read the data.
                *p = block.dr.read().dr().bits();
                if T::event_flag(Event::BTF) {
                    T::ack(false);
                    let (_, p) = enumerate.next().unwrap();
                    *p = block.dr.read().dr().bits();
                }
            } else if remain == 1 {
                T::ack(false);
                // EV7: RxNE=1, read DR to clear this bit.
                EventFuture::<T>::new(EnumSet::empty() | Event::RXNE).await?;
                // Read the data.
                *p = block.dr.read().dr().bits();
            }
        }

        T::stop();

        Ok(buf.len())
    }

    pub async fn write(&mut self, address: u8, buf: &[u8]) -> Result<usize, Error> {
        // If the bus is busy, exit.
        T::clear_pos();
        T::start();
        // SB=1: cleared by reading SR1 then writing the data register.
        EventFuture::<T>::new(EnumSet::empty() | Event::SB).await?;

        T::transmit(address << 1);

        // ADDR=1: cleared by reading SR1 then SR2.
        EventFuture::<T>::new(EnumSet::empty() | Event::ADD)
            .await
            .inspect_err(|_| {
                T::event_clear(Event::AF);
                T::stop();
            })?;
        T::event_clear(Event::ADD);

        // The TRA bit indicates whether the master is in receiver or transmitter mode.
        let mut iter = buf.iter();
        if let Some(d) = iter.next() {
            // EV8_1: TxE=1, shift register empty, data register empty, write Data1 to DR.
            EventFuture::<T>::new(EnumSet::empty() | Event::TXE).await?;
            T::transmit(*d);
        }

        // Then send the remaining data.
        for t in iter {
            // EV8: TxE=1, shift register not empty, data register empty, write Data2 to DR,
            // this bit is cleared.
            EventFuture::<T>::new(EnumSet::empty() | Event::TXE | Event::BTF)
                .await
                .inspect_err(|_| {
                    T::stop();
                })?;

            T::transmit(*t);
        }

        // EV8_2: TxE=1, BTF=1, write the Stop bit; when hardware sends the Stop bit,
        // TxE and BTF are cleared.
        EventFuture::<T>::new(EnumSet::empty() | Event::TXE)
            .await
            .inspect_err(|_| {
                T::stop();
            })?;

        EventFuture::<T>::new(EnumSet::empty() | Event::TXE)
            .await
            .inspect_err(|_| {
                T::stop();
            })?;

        EventFuture::<T>::new(EnumSet::empty() | Event::BTF)
            .await
            .inspect_err(|_| {
                T::stop();
            })?;

        T::stop();

        Ok(buf.len())
    }
}

impl embedded_hal_async::i2c::Error for Error {
    fn kind(&self) -> embedded_hal_async::i2c::ErrorKind {
        match *self {
            Self::Address => embedded_hal_async::i2c::ErrorKind::NoAcknowledge(
                embedded_hal_async::i2c::NoAcknowledgeSource::Address,
            ),
            Self::Busy => embedded_hal_async::i2c::ErrorKind::Other,
            Self::PClock => embedded_hal_async::i2c::ErrorKind::Other,
            Self::RX => embedded_hal_async::i2c::ErrorKind::Other,
            Self::SpeedMode => embedded_hal_async::i2c::ErrorKind::Other,
            Self::Start => embedded_hal_async::i2c::ErrorKind::Other,
            Self::Stop => embedded_hal_async::i2c::ErrorKind::Other,
            Self::Tx => embedded_hal_async::i2c::ErrorKind::Other,
            Self::Acknowledge => embedded_hal_async::i2c::ErrorKind::NoAcknowledge(
                embedded_hal_async::i2c::NoAcknowledgeSource::Unknown,
            ),
            Self::ArbitrationLost => embedded_hal_async::i2c::ErrorKind::ArbitrationLoss,
            Self::Bus => embedded_hal_async::i2c::ErrorKind::Bus,
            Self::Overrun => embedded_hal_async::i2c::ErrorKind::Overrun,
            Self::Timeout => embedded_hal_async::i2c::ErrorKind::Other,
        }
    }
}

#[cfg(feature = "embassy")]
impl<'d, T: Instance> embedded_hal_async::i2c::ErrorType for Master<'d, T, Async> {
    type Error = Error;
}

// Note: the async I2C transfer for Master is not yet implemented (the original
// `embedded_hal_async::i2c::I2c` impl was full of `todo!()` and would panic on
// call). For safety this trait is intentionally not implemented, so the
// unsupported capability is caught at compile time rather than panicking at
// runtime. To support async I2C, implement the logic below.
