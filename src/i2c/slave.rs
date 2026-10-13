use super::{Error, Event, Instance};
use crate::delay::wait_for_true_timeout_block;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;

const WAIT_FLAG_TIMEOUT: usize = 10000;

/// Slave role.
///
/// Slave flow:
/// 1. Wait for the master to address this slave (`ADDR` set); read SR1 + SR2 to clear it.
/// 2. Decide whether the master wants to read or write based on the `TRA` bit.
/// 3. The transfer ends when `STOPF` (master stops) or `AF` (master NACK) is set.
pub struct Slave<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,
}

impl<'d, T: Instance, M: Mode> Slave<'d, T, M> {
    pub(super) fn new() -> Self {
        Self { _t: PhantomData }
    }

    /// Whether the bus is busy.
    #[inline]
    pub fn is_busy(&self) -> bool {
        T::busy()
    }

    /// Whether a general-call address was received.
    #[inline]
    pub fn is_general_call(&self) -> bool {
        T::is_general_call()
    }
}

impl<'d, T: Instance> Slave<'d, T, Blocking> {
    /// Wait for the master to address this slave; returns whether the master wants to
    /// read (true) or write (false) this time.
    ///
    /// Master writes to slave → `TRA=0`; master reads from slave → `TRA=1`.
    pub fn wait_addressed(&mut self) -> Result<bool, Error> {
        if wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::ADD)).is_err() {
            return Err(Error::Timeout);
        }

        // Read SR1 + SR2 to clear ADDR.
        T::clear_addr();

        Ok(T::is_transmitter())
    }

    /// Receive data from the master (master writes, slave receives).
    ///
    /// Ends when a stop condition is received or the buffer is full; returns the number
    /// of bytes actually received.
    pub fn read_block(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let len = buf.len();
        let mut cnt = 0;
        for v in buf.iter_mut() {
            // Reply NACK on the last byte to tell the master to stop.
            if cnt + 1 == len {
                T::ack(false);
            } else {
                T::ack(true);
            }

            if wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::RXNE))
                .is_err()
            {
                return Err(Error::Timeout);
            }

            *v = T::block().dr.read().dr().bits();
            cnt += 1;

            // The master sent a stop condition early.
            if T::event_flag(Event::STOPF) {
                Self::clear_stop();
                return Ok(cnt);
            }
        }
        Ok(cnt)
    }

    /// Send data to the master (master reads, slave transmits).
    pub fn write_block(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let mut cnt = 0;
        for v in buf.iter() {
            if wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::TXE))
                .is_err()
            {
                return Err(Error::Timeout);
            }

            T::transmit(*v);
            cnt += 1;

            // A NACK from the master means it will not receive any more.
            if T::event_flag(Event::AF) {
                T::event_clear(Event::AF);
                break;
            }

            if T::event_flag(Event::STOPF) {
                Self::clear_stop();
                break;
            }
        }
        Ok(cnt)
    }

    /// Clear the stop-condition flag (STOPF): read SR1 then write CR1.
    fn clear_stop() {
        let _ = T::block().sr1.read().bits();
        T::block().cr1.modify(|_, w| w.pe().set_bit());
    }
}
