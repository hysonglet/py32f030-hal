use enumset::EnumSet;

use super::{future::EventFuture, Error, Event, Instance};
use crate::{
    clock::peripheral::PeripheralInterrupt,
    mode::{Async, Blocking, Mode},
};
use core::marker::PhantomData;

/// Master 角色
pub struct Master<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,
}

impl<'d, T: Instance, M: Mode> Master<'d, T, M> {
    pub(super) fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<'d, T: Instance> Master<'d, T, Blocking> {
    pub fn write_block(&self, address: u8, buf: &[u8]) -> Result<usize, Error> {
        T::master_transmit_block(address, buf)
    }

    pub fn clear_errors(&self) {
        // T::soft_reset()
    }

    pub fn read_block(&self, address: u8, buf: &mut [u8]) -> Result<usize, Error> {
        T::master_receive_block(address, buf)
    }
}

////////////////////////////////////////////////////////////////////////////////
impl<'d, T: Instance> embedded_hal::blocking::i2c::Write for Master<'d, T, Blocking> {
    type Error = Error;
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write_block(address, bytes)
            .map_or_else(Err, |_| Ok(()))
    }
}

impl<'d, T: Instance> embedded_hal::blocking::i2c::Read for Master<'d, T, Blocking> {
    type Error = Error;
    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.read_block(address, buffer)
            .map_or_else(Err, |_| Ok(()))
    }
}

impl<'d, T: Instance> embedded_hal::blocking::i2c::WriteRead for Master<'d, T, Blocking> {
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

    pub async fn write(&self, address: u8, buf: &[u8]) -> Result<usize, Error> {
        // 如果总线处于busy状态，则退出

        T::id().enable_interrupt();
        T::clear_pos();
        T::start();
        // SB=1，通过读 SR1，再向 DR 寄存器写数据，实现对该位的清零
        EventFuture::<T>::new(EnumSet::empty() | Event::SB).await?;

        T::transmit(address << 1);

        // ADDR=1，通过读 SR1，再读 SR2，实现对该位的清零
        EventFuture::<T>::new(EnumSet::empty() | Event::ADD)
            .await
            .map_err(|e| {
                T::clear_answer_faild();
                T::stop();
                e
            })?;

        T::clear_address_flag();

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
                .map_err(|e| {
                    T::stop();
                    e
                })?;

            T::transmit(*t);
        }

        // EV8_2：TxE=1, BTF=1, 写 Stop 位寄存器，当硬件发出 Stop 位时，TxE 和 BTF 被清零
        EventFuture::<T>::new(EnumSet::empty() | Event::TXE)
            .await
            .map_err(|e| {
                T::stop();
                e
            })?;

        EventFuture::<T>::new(EnumSet::empty() | Event::TXE)
            .await
            .map_err(|e| {
                T::stop();
                e
            })?;

        EventFuture::<T>::new(EnumSet::empty() | Event::BTF)
            .await
            .map_err(|e| {
                T::stop();
                e
            })?;

        T::stop();

        T::id().disable_interrupt();
        Ok(buf.len())
    }
}
