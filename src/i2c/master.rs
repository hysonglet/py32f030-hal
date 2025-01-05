#[cfg(feature = "embassy")]
use enumset::EnumSet;

#[cfg(feature = "embassy")]
use super::future::EventFuture;
use super::hal::sealed::WAIT_FLAG_TIMEOUT;
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

/// Master 角色
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
        // SB=1，通过读 SR1，再向 DR 寄存器写数据，实现对该位的清零
        wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::SB)).map_err(
            |_| {
                T::event_clear(Event::ARLO);
                Error::Start
            },
        )?;

        T::transmit(address << 1);

        // ADDR=1，通过读 SR1，再读 SR2，实现对该位的清零
        wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::ADD)).map_err(
            |_| {
                // Self::debug();
                // 清除 af 置位
                T::event_clear(Event::AF);
                T::stop();
                Error::Address
            },
        )?;
        T::event_clear(Event::ADD);

        // TRA 位指示主设备是在接收器模式还是发送器模式。

        let mut iter = buf.iter();
        if let Some(d) = iter.next() {
            // EV8_1：TxE=1, shift 寄存器 empty，数据寄存器 empty，向 DR 寄存器写 Data1
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::TXE))
                .map_err(|_| Error::Tx)?;
            T::transmit(*d);
        }

        // 接着将后面的数据发送出去
        for t in iter {
            // EV8：TxE=1, shift 寄存器不 empty，数据寄存器 empty，向 DR 寄存器写 Data2，该位被清零
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::TXE)).map_err(
                |_| {
                    T::stop();
                    Error::Tx
                },
            )?;
            T::transmit(*t);
        }

        // EV8_2：TxE=1, BTF=1, 写 Stop 位寄存器，当硬件发出 Stop 位时，TxE 和 BTF 被清零
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

////////////////////////////////////////////////////////////////////////////////

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

////////

#[cfg(feature = "embassy")]
impl<'d, T: Instance> Master<'d, T, Async> {
    pub async fn read(&self, address: u8, buf: &mut [u8]) -> Result<usize, Error> {
        let block = T::block();

        T::start();

        // EV5：SB=1, 先读 SR1 寄存器，再写 DR 寄存器，清零该位
        EventFuture::<T>::new(EnumSet::empty() | Event::SB).await?;

        T::transmit((address << 1) | 1);

        // EV6：ADDR，先读 SR1，再读 SR2，清零该位
        EventFuture::<T>::new(EnumSet::empty() | Event::ADD).await?;

        let len = buf.len();

        let mut enumerate = buf.iter_mut().enumerate();
        while let Some((idx, p)) = enumerate.next() {
            let remain = len - idx;
            if remain > 2 {
                T::ack(true);
                // EV7：RxNE=1, 读 DR 寄存器清零该位
                EventFuture::<T>::new(EnumSet::empty() | Event::RXNE).await?;

                // 读取数据
                *p = block.dr.read().dr().bits();

                if T::event_flag(Event::BTF) {
                    T::ack(true);
                    let (_, p) = enumerate.next().unwrap();
                    *p = block.dr.read().dr().bits();
                }
            } else if remain == 2 {
                T::ack(false);
                // EV7：RxNE=1, 读 DR 寄存器清零该位
                EventFuture::<T>::new(EnumSet::empty() | Event::RXNE).await?;
                // 读取数据
                *p = block.dr.read().dr().bits();
                if T::event_flag(Event::BTF) {
                    T::ack(false);
                    let (_, p) = enumerate.next().unwrap();
                    *p = block.dr.read().dr().bits();
                }
            } else if remain == 1 {
                T::ack(false);
                // EV7：RxNE=1, 读 DR 寄存器清零该位
                // EV7：RxNE=1, 读 DR 寄存器清零该位
                EventFuture::<T>::new(EnumSet::empty() | Event::RXNE).await?;
                // 读取数据
                *p = block.dr.read().dr().bits();
            }
        }

        T::stop();

        Ok(buf.len())
    }

    pub async fn write(&mut self, address: u8, buf: &[u8]) -> Result<usize, Error> {
        // 如果总线处于busy状态，则退出
        T::clear_pos();
        T::start();
        // SB=1，通过读 SR1，再向 DR 寄存器写数据，实现对该位的清零
        EventFuture::<T>::new(EnumSet::empty() | Event::SB).await?;

        T::transmit(address << 1);

        // ADDR=1，通过读 SR1，再读 SR2，实现对该位的清零
        EventFuture::<T>::new(EnumSet::empty() | Event::ADD)
            .await
            .inspect_err(|_| {
                T::event_clear(Event::AF);
                T::stop();
            })?;
        T::event_clear(Event::ADD);

        // TRA 位指示主设备是在接收器模式还是发送器模式。
        let mut iter = buf.iter();
        if let Some(d) = iter.next() {
            // EV8_1：TxE=1, shift 寄存器 empty，数据寄存器 empty，向 DR 寄存器写 Data1
            EventFuture::<T>::new(EnumSet::empty() | Event::TXE).await?;
            T::transmit(*d);
        }

        // 接着将后面的数据发送出去
        for t in iter {
            // EV8：TxE=1, shift 寄存器不 empty，数据寄存器 empty，向 DR 寄存器写 Data2，该位被清零
            EventFuture::<T>::new(EnumSet::empty() | Event::TXE | Event::BTF)
                .await
                .inspect_err(|_| {
                    T::stop();
                })?;

            T::transmit(*t);
        }

        // EV8_2：TxE=1, BTF=1, 写 Stop 位寄存器，当硬件发出 Stop 位时，TxE 和 BTF 被清零
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
        }
    }
}

#[cfg(feature = "embassy")]
impl<'d, T: Instance> embedded_hal_async::i2c::ErrorType for Master<'d, T, Async> {
    type Error = Error;
}

#[cfg(feature = "embassy")]
impl<'d, T: Instance> embedded_hal_async::i2c::I2c for Master<'d, T, Async> {
    async fn read(&mut self, _address: u8, _read: &mut [u8]) -> Result<(), Self::Error> {
        todo!()
    }

    async fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn write(&mut self, _address: u8, _write: &[u8]) -> Result<(), Self::Error> {
        // self.write(address, write).await.map_or_else((), |e| e)
        todo!()
    }
}
