#[cfg(feature = "embassy")]
mod future;
mod hal;
mod types;

use crate::clock::peripheral::{
    PeripheralClockIndex, PeripheralIdToClockIndex, PeripheralInterrupt,
};
use crate::macro_def::impl_sealed_peripheral_id;
#[cfg(feature = "embassy")]
use crate::mode::Async;
use crate::mode::{Blocking, Mode};
use crate::syscfg::{syscfg, DmaChannelMap};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};
use enumset::EnumSet;
#[cfg(feature = "embassy")]
use future::EventFuture;
pub use types::*;

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

/// 串口号定义
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Id {
    DMA,
}

impl_sealed_peripheral_id!(DMA, DMA);

impl PeripheralIdToClockIndex for Id {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            Self::DMA => PeripheralClockIndex::DMA,
        }
    }
}

/// 通道 id
#[derive(PartialEq, Clone, Copy)]
pub enum Channel {
    Channel1 = 0,
    Channel2 = 1,
    Channel3 = 2,
}

impl PeripheralInterrupt for Channel {
    fn interrupt(&self) -> crate::pac::interrupt {
        match *self {
            Self::Channel1 => PY32f030xx_pac::interrupt::DMA_CHANNEL1,
            Self::Channel2 | Self::Channel3 => PY32f030xx_pac::interrupt::DMA_CHANNEL2_3,
        }
    }
}

pub struct Config {
    diretion: Direction,
    prioritie: Priorities,
    mode: RepeatMode,
    memInc: bool,
    periphInc: bool,
    periphDataSize: Burst,
    memDataSize: Burst,
    periphAddr: u32,
    memAddr: u32,
}

impl Config {
    // 新建配置结构体
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        diretion: Direction,
        prioritie: Priorities,
        mode: RepeatMode,
        memInc: bool,
        periphInc: bool,
        periphDataSize: Burst,
        memDataSize: Burst,
        memAddr: u32,
        periphAddr: u32,
    ) -> Self {
        Self {
            diretion,
            prioritie,
            mode,
            memInc,
            periphInc,
            periphDataSize,
            memDataSize,
            memAddr,
            periphAddr,
        }
    }

    /// 新建内存到内存的配置
    pub fn new_mem2mem(
        src_addr: u32,
        src_inc: bool,
        dst_addr: u32,
        dst_inc: bool,
        priorite: Priorities,
        mode: RepeatMode,
        burst: Burst,
    ) -> Self {
        Self::new(
            Direction::MemoryToMemory,
            priorite,
            mode,
            src_inc,
            dst_inc,
            burst,
            burst,
            src_addr,
            dst_addr,
        )
    }

    /// 新建内存到外设的配置
    #[allow(clippy::too_many_arguments)]
    pub fn new_mem2periph(
        src_addr: u32,
        src_inc: bool,
        src_burst: Burst,
        dst_addr: u32,
        dst_inc: bool,
        dst_burst: Burst,
        priorite: Priorities,
        mode: RepeatMode,
    ) -> Config {
        Self::new(
            Direction::MemoryToPeriph,
            priorite,
            mode,
            src_inc,
            dst_inc,
            src_burst,
            dst_burst,
            src_addr,
            dst_addr,
        )
    }

    /// 新建外设到内存的配置
    #[allow(clippy::too_many_arguments)]
    pub fn new_periph2mem(
        src_addr: u32,
        src_inc: bool,
        src_burst: Burst,
        dst_addr: u32,
        dst_inc: bool,
        burst: Burst,
        priorite: Priorities,
        mode: RepeatMode,
    ) -> Config {
        Self {
            diretion: Direction::PeriphToMemory,
            prioritie: priorite,
            mode,
            memDataSize: burst,
            periphDataSize: src_burst,
            memAddr: dst_addr,
            periphAddr: src_addr,
            memInc: dst_inc,
            periphInc: src_inc,
        }
    }
}

pub struct AnyDma<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _mode: PhantomData<M>,
}

impl<'d, T: Instance, M: Mode> Drop for AnyDma<'d, T, M> {
    fn drop(&mut self) {
        T::id().clock().close();
    }
}

impl<'d, T: Instance, M: Mode> AnyDma<'d, T, M> {
    pub fn new(_dma: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(_dma);

        T::id().clock().open();

        Self {
            _t: PhantomData,
            _mode: PhantomData,
        }
    }

    pub fn split(&mut self) -> [DmaChannel<T, M>; 3] {
        [
            DmaChannel::new(Channel::Channel1),
            DmaChannel::new(Channel::Channel2),
            DmaChannel::new(Channel::Channel3),
        ]
    }
}

pub struct DmaChannel<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _mode: PhantomData<M>,
    channel: Channel,
}

impl<'d, T: Instance, M: Mode> DmaChannel<'d, T, M> {
    /// 新建
    pub(super) fn new(channel: Channel) -> Self {
        Self {
            _t: PhantomData,
            _mode: PhantomData,
            channel,
        }
    }

    /// 绑定通道到外设
    pub fn bind(&mut self, map: DmaChannelMap) {
        syscfg::set_dma_channel_map(self.channel, map);
    }

    /// 使能dma快速响应
    pub fn en_fast_response(&mut self, en: bool) {
        syscfg::en_dma_channel_fast_response(self.channel, en);
    }

    /// 配置dma通道
    #[inline]
    pub fn config(&mut self, config: Config) {
        T::config(self.channel, config)
    }

    /// 开始dma
    #[inline]
    pub fn start(&mut self) {
        T::enable(self.channel, true);
    }

    /// 停止或取消dma
    #[inline]
    pub fn stop(&mut self) {
        T::enable(self.channel, false);
    }

    /// 返回剩余的数量
    pub fn remain(&self) -> u16 {
        T::remain_count(self.channel)
    }
}

impl<'d, T: Instance> DmaChannel<'d, T, Blocking> {
    /// 返回是否结束
    pub fn is_finish(&self) -> bool {
        T::event_flag(self.channel, Event::TCIF)
    }

    /// 返回是否发生错误
    pub fn is_error(&self) -> bool {
        T::event_flag(self.channel, Event::TEIF)
    }

    /// 清除状态标志
    pub fn clear_flag(&mut self, events: EnumSet<Event>) {
        for e in events {
            T::event_clear(self.channel, e);
        }
    }

    /// 等待传输完成
    pub fn wait_complet(&self) -> Result<(), Error> {
        while !T::event_flag(self.channel, Event::TCIF) {
            // 检查是否出错
            if T::event_flag(self.channel, Event::TEIF) {
                T::event_clear(self.channel, Event::TEIF);
                return Err(Error::Others);
            }
        }
        T::event_clear(self.channel, Event::TCIF);
        Ok(())
    }

    /// 等待半完成
    pub fn wait_half_complet(&self) -> Result<(), Error> {
        while !T::event_flag(self.channel, Event::HTIF) {
            // 检查是否出错
            if T::event_flag(self.channel, Event::TEIF) {
                T::event_clear(self.channel, Event::TEIF);
                return Err(Error::Others);
            }
        }
        T::event_clear(self.channel, Event::HTIF);
        Ok(())
    }
}

#[cfg(feature = "embassy")]
impl<'d, T: Instance> DmaChannel<'d, T, Async> {
    /// 等待完成
    pub async fn wait_complet(&self) -> Result<(), Error> {
        if EventFuture::<T>::new(self.channel, Event::TCIF | Event::TEIF).await != Event::TCIF {
            return Err(Error::Others);
        }

        Ok(())
    }

    /// 等待半完成
    pub async fn wait_half_complet(&self) -> Result<(), Error> {
        if EventFuture::<T>::new(self.channel, Event::HTIF | Event::TEIF).await != Event::HTIF {
            return Err(Error::Others);
        }

        Ok(())
    }
}
