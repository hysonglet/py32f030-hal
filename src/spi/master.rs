use super::{Error, Instance};
use crate::delay::wait_for_true_timeout_block;
use crate::dma::{self, DmaChannel};
use crate::gpio::AnyPin;
use crate::mcu::peripherals::DMA;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;
use embassy_hal_internal::PeripheralRef;

const TIMEOUT: usize = 10000;

/// Master 角色
pub struct Master<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,

    // 引脚
    _sck: PeripheralRef<'d, AnyPin>,
    _mosi: Option<PeripheralRef<'d, AnyPin>>,
    _miso: Option<PeripheralRef<'d, AnyPin>>,
    _nss: Option<PeripheralRef<'d, AnyPin>>,

    tx_dma: Option<DmaChannel<'d, DMA, M>>,
    rx_dma: Option<DmaChannel<'d, DMA, M>>,
}

impl<'d, T: Instance, M: Mode> Master<'d, T, M> {
    pub(super) fn new(
        // 引脚
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

    /// 是否配置了发送 DMA 通道
    #[inline]
    pub fn has_tx_dma(&self) -> bool {
        self.tx_dma.is_some()
    }

    /// 是否配置了接收 DMA 通道
    #[inline]
    pub fn has_rx_dma(&self) -> bool {
        self.rx_dma.is_some()
    }
}

impl<'d, T: Instance, M: Mode> Master<'d, T, M> {
    /// 总线是否忙
    #[inline]
    pub fn is_busy(&self) -> bool {
        T::is_busy()
    }

    /// 是否发生溢出
    #[inline]
    pub fn is_overflow(&self) -> bool {
        T::is_overflow()
    }

    /// 是否发生模式错误（MODF）
    #[inline]
    pub fn is_mode_fault(&self) -> bool {
        T::is_mode_fault()
    }

    /// 清除错误标志（OVR / MODF）
    #[inline]
    pub fn clear_errors(&self) {
        T::clear_errors()
    }

    /// 等待总线空闲
    pub fn flush(&self) -> Result<(), Error> {
        if wait_for_true_timeout_block(TIMEOUT, || !T::is_busy()).is_err() {
            return Err(Error::Timeout);
        }
        Ok(())
    }

    /// 返回当前 SPI 总线频率
    #[inline]
    pub fn get_baud_rate(&self) -> u32 {
        T::get_baud_rate()
    }
}

impl<'d, T: Instance> Master<'d, T, Blocking> {
    /// 阻塞写入（丢弃读到的数据）
    pub fn write_block(&mut self, buf: &[u8]) -> Result<usize, Error> {
        // master 模式如果没有配置 mosi 引脚，则无法发送数据
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

    /// 阻塞读取（发送空字节产生时钟）
    pub fn read_block(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        // master 模式如果没有配置 miso 引脚，则无法接收数据
        if self._miso.is_none() {
            return Err(Error::Read);
        }
        let mut cnt: usize = 0;
        for v in buf.iter_mut() {
            // 发送哑元以产生时钟
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

    /// 全双工传输：同时收发，`read` 为收到的字节，`write` 为要发送的字节
    ///
    /// 当 `read` 比 `write` 长时，多余部分发送 0x00 填充；
    /// 当 `write` 比 `read` 长时，多余收到的字节被丢弃。
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

    /// 原地全双工传输：缓冲区既是发送内容也被接收内容覆盖
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

    ////////////////// DMA //////////////////

    /// DMA 方式写入
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

    /// DMA 方式读取（需同时配置发送 DMA 以产生时钟）
    pub fn read_dma_block(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        let (tx_map, rx_map) = T::id().dma_channel_map();

        if let (Some(tx_dma), Some(rx_dma)) = (&mut self.tx_dma, &mut self.rx_dma) {
            let _tx_guard = drop_move::DropGuard::new(|| T::tx_dma_enable(false));
            let _rx_guard = drop_move::DropGuard::new(|| T::rx_dma_enable(false));

            // 发送哑元以产生时钟
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

    /// DMA 方式全双工传输
    pub fn transfer_dma_block(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Error> {
        let (tx_map, rx_map) = T::id().dma_channel_map();

        if let (Some(tx_dma), Some(rx_dma)) = (&mut self.tx_dma, &mut self.rx_dma) {
            let _tx_guard = drop_move::DropGuard::new(|| T::tx_dma_enable(false));
            let _rx_guard = drop_move::DropGuard::new(|| T::rx_dma_enable(false));

            let len = core::cmp::max(read.len(), write.len());
            let write_ptr = if write.is_empty() {
                // 无发送数据：用静态哑元填充
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

////////////////////////////// embedded-hal 1.0 //////////////////////////////

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

////////////////////////////// embedded-hal 0.2 //////////////////////////////

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
