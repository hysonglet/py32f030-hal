use super::{Error, Instance};
use crate::delay::wait_for_true_timeout_block;
use crate::dma::{self, DmaChannel};
use crate::gpio::AnyPin;
use crate::mcu::peripherals::DMA;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;
use embassy_hal_internal::PeripheralRef;

const TIMEOUT: usize = 10000;

/// Master role.
pub struct Master<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,

    // Pins
    _sck: PeripheralRef<'d, AnyPin>,
    _mosi: Option<PeripheralRef<'d, AnyPin>>,
    _miso: Option<PeripheralRef<'d, AnyPin>>,
    _nss: Option<PeripheralRef<'d, AnyPin>>,

    tx_dma: Option<DmaChannel<'d, DMA, M>>,
    rx_dma: Option<DmaChannel<'d, DMA, M>>,
}

impl<'d, T: Instance, M: Mode> Master<'d, T, M> {
    pub(super) fn new(
        // Pins
        _sck: PeripheralRef<'d, AnyPin>,
        _mosi: Option<PeripheralRef<'d, AnyPin>>,
        _miso: Option<PeripheralRef<'d, AnyPin>>,
        _nss: Option<PeripheralRef<'d, AnyPin>>,
        tx_dma: Option<DmaChannel<'d, DMA, M>>,
        rx_dma: Option<DmaChannel<'d, DMA, M>>,
    ) -> Self {
        Self {
            _t: PhantomData,
            _sck,
            _miso,
            _mosi,
            _nss,
            tx_dma,
            rx_dma,
        }
    }

    /// Whether a transmit DMA channel is configured.
    #[inline]
    pub fn has_tx_dma(&self) -> bool {
        self.tx_dma.is_some()
    }

    /// Whether a receive DMA channel is configured.
    #[inline]
    pub fn has_rx_dma(&self) -> bool {
        self.rx_dma.is_some()
    }
}

impl<'d, T: Instance, M: Mode> Master<'d, T, M> {
    /// Whether the bus is busy.
    #[inline]
    pub fn is_busy(&self) -> bool {
        T::is_busy()
    }

    /// Whether an overrun occurred.
    #[inline]
    pub fn is_overflow(&self) -> bool {
        T::is_overflow()
    }

    /// Whether a mode fault (MODF) occurred.
    #[inline]
    pub fn is_mode_fault(&self) -> bool {
        T::is_mode_fault()
    }

    /// Clear error flags (OVR / MODF).
    #[inline]
    pub fn clear_errors(&self) {
        T::clear_errors()
    }

    /// Wait for the bus to become idle.
    pub fn flush(&self) -> Result<(), Error> {
        if wait_for_true_timeout_block(TIMEOUT, || !T::is_busy()).is_err() {
            return Err(Error::Timeout);
        }
        Ok(())
    }

    /// Return the current SPI bus frequency.
    #[inline]
    pub fn get_baud_rate(&self) -> u32 {
        T::get_baud_rate()
    }
}

impl<'d, T: Instance> Master<'d, T, Blocking> {
    /// Blocking write (discard the data read back).
    pub fn write_block(&mut self, buf: &[u8]) -> Result<usize, Error> {
        // In master mode, data cannot be sent without a MOSI pin.
        if self._mosi.is_none() {
            return Err(Error::Write);
        }
        for v in buf.iter() {
            if wait_for_true_timeout_block(TIMEOUT, || T::tx_empty()).is_err() {
                return Err(Error::Timeout);
            }
            T::data_write(*v as u16)
        }
        self.flush()?;
        Ok(buf.len())
    }

    /// Blocking read (transmit dummy bytes to generate the clock).
    pub fn read_block(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        // In master mode, data cannot be received without a MISO pin.
        if self._miso.is_none() {
            return Err(Error::Read);
        }
        let mut cnt: usize = 0;
        for v in buf.iter_mut() {
            // Send a dummy to generate the clock.
            if wait_for_true_timeout_block(TIMEOUT, || T::tx_empty()).is_err() {
                return Err(Error::Timeout);
            }
            T::data_write(0);

            if wait_for_true_timeout_block(TIMEOUT, || T::rx_not_empty()).is_err() {
                return Err(Error::Timeout);
            }
            *v = T::data_read() as u8;
            cnt += 1;
        }
        Ok(cnt)
    }

    /// Full-duplex transfer: send and receive simultaneously; `read` holds received bytes, `write` holds bytes to send.
    ///
    /// When `read` is longer than `write`, the extra part is filled with 0x00;
    /// when `write` is longer than `read`, the extra received bytes are discarded.
    pub fn transfer_block(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        if self._mosi.is_none() || self._miso.is_none() {
            return Err(Error::Write);
        }

        let len = core::cmp::max(read.len(), write.len());
        for i in 0..len {
            let tx = write.get(i).copied().unwrap_or(0);

            if wait_for_true_timeout_block(TIMEOUT, || T::tx_empty()).is_err() {
                return Err(Error::Timeout);
            }
            T::data_write(tx as u16);

            if wait_for_true_timeout_block(TIMEOUT, || T::rx_not_empty()).is_err() {
                return Err(Error::Timeout);
            }
            let rx = T::data_read() as u8;
            if let Some(r) = read.get_mut(i) {
                *r = rx;
            }
        }

        self.flush()?;
        Ok(())
    }

    /// In-place full-duplex transfer: the buffer is both sent and overwritten by received data.
    pub fn transfer_in_place_block(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        if self._mosi.is_none() || self._miso.is_none() {
            return Err(Error::Write);
        }

        for v in buf.iter_mut() {
            if wait_for_true_timeout_block(TIMEOUT, || T::tx_empty()).is_err() {
                return Err(Error::Timeout);
            }
            T::data_write(*v as u16);

            if wait_for_true_timeout_block(TIMEOUT, || T::rx_not_empty()).is_err() {
                return Err(Error::Timeout);
            }
            *v = T::data_read() as u8;
        }

        self.flush()?;
        Ok(())
    }


    /// Write via DMA.
    pub fn write_dma_block(&mut self, buf: &[u8]) -> Result<(), Error> {
        let (tx_map, _rx_map) = T::id().dma_channel_map();

        if let Some(dma) = &mut self.tx_dma {
            let _guard = drop_move::DropGuard::new(|| T::tx_dma_enable(false));

            dma.clear_flag(enumset::EnumSet::all());
            dma.config(dma::Config::new_mem2periph(
                buf.as_ptr() as u32,
                true,
                dma::Burst::Single,
                T::block().dr.as_ptr() as u32,
                false,
                dma::Burst::Single,
                dma::Priorities::Medium,
                dma::RepeatMode::OneTime(buf.len() as u16),
            ));
            dma.bind(tx_map);
            dma.start();

            T::tx_dma_enable(true);
            dma.wait_complet().map_err(|_| Error::Dma)?;
            self.flush()?;
            Ok(())
        } else {
            Err(Error::Dma)
        }
    }

    /// Read via DMA (requires a transmit DMA channel to generate the clock).
    pub fn read_dma_block(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        let (tx_map, rx_map) = T::id().dma_channel_map();

        if let (Some(tx_dma), Some(rx_dma)) = (&mut self.tx_dma, &mut self.rx_dma) {
            let _tx_guard = drop_move::DropGuard::new(|| T::tx_dma_enable(false));
            let _rx_guard = drop_move::DropGuard::new(|| T::rx_dma_enable(false));

            // Send a dummy to generate the clock.
            static DUMMY: u8 = 0;
            tx_dma.clear_flag(enumset::EnumSet::all());
            tx_dma.config(dma::Config::new_mem2periph(
                &DUMMY as *const u8 as u32,
                false,
                dma::Burst::Single,
                T::block().dr.as_ptr() as u32,
                false,
                dma::Burst::Single,
                dma::Priorities::Medium,
                dma::RepeatMode::OneTime(buf.len() as u16),
            ));
            tx_dma.bind(tx_map);

            rx_dma.clear_flag(enumset::EnumSet::all());
            rx_dma.config(dma::Config::new_periph2mem(
                T::block().dr.as_ptr() as u32,
                false,
                dma::Burst::Single,
                buf.as_mut_ptr() as u32,
                true,
                dma::Burst::Single,
                dma::Priorities::Medium,
                dma::RepeatMode::OneTime(buf.len() as u16),
            ));
            rx_dma.bind(rx_map);

            rx_dma.start();
            tx_dma.start();

            T::rx_dma_enable(true);
            T::tx_dma_enable(true);

            rx_dma.wait_complet().map_err(|_| Error::Dma)?;
            self.flush()?;
            Ok(())
        } else {
            Err(Error::Dma)
        }
    }

    /// Full-duplex transfer via DMA.
    pub fn transfer_dma_block(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        let (tx_map, rx_map) = T::id().dma_channel_map();

        if let (Some(tx_dma), Some(rx_dma)) = (&mut self.tx_dma, &mut self.rx_dma) {
            let _tx_guard = drop_move::DropGuard::new(|| T::tx_dma_enable(false));
            let _rx_guard = drop_move::DropGuard::new(|| T::rx_dma_enable(false));

            let len = core::cmp::max(read.len(), write.len());
            let write_ptr = if write.is_empty() {
                // No data to send: fill with a static dummy.
                static DUMMY: u8 = 0;
                &DUMMY as *const u8 as u32
            } else {
                write.as_ptr() as u32
            };
            let write_inc = !write.is_empty();

            tx_dma.clear_flag(enumset::EnumSet::all());
            tx_dma.config(dma::Config::new_mem2periph(
                write_ptr,
                write_inc,
                dma::Burst::Single,
                T::block().dr.as_ptr() as u32,
                false,
                dma::Burst::Single,
                dma::Priorities::Medium,
                dma::RepeatMode::OneTime(len as u16),
            ));
            tx_dma.bind(tx_map);

            rx_dma.clear_flag(enumset::EnumSet::all());
            rx_dma.config(dma::Config::new_periph2mem(
                T::block().dr.as_ptr() as u32,
                false,
                dma::Burst::Single,
                read.as_mut_ptr() as u32,
                true,
                dma::Burst::Single,
                dma::Priorities::Medium,
                dma::RepeatMode::OneTime(read.len().min(len) as u16),
            ));
            rx_dma.bind(rx_map);

            rx_dma.start();
            tx_dma.start();

            T::rx_dma_enable(true);
            T::tx_dma_enable(true);

            tx_dma.wait_complet().map_err(|_| Error::Dma)?;
            self.flush()?;
            Ok(())
        } else {
            Err(Error::Dma)
        }
    }
}


impl embedded_hal::spi::Error for Error {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        embedded_hal::spi::ErrorKind::Other
    }
}

impl<'d, T: Instance> embedded_hal::spi::ErrorType for Master<'d, T, Blocking> {
    type Error = Error;
}

impl<'d, T: Instance> embedded_hal::spi::SpiBus<u8> for Master<'d, T, Blocking> {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.read_block(words)?;
        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.write_block(words)?;
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.transfer_block(read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.transfer_in_place_block(words)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Master::flush(self)
    }
}


impl<'d, T: Instance> embedded_hal_027::blocking::spi::Transfer<u8> for Master<'d, T, Blocking> {
    type Error = Error;
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.transfer_in_place_block(words)?;
        Ok(words)
    }
}

impl<'d, T: Instance> embedded_hal_027::blocking::spi::Write<u8> for Master<'d, T, Blocking> {
    type Error = Error;
    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        self.write_block(words)?;
        Ok(())
    }
}
