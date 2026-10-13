#[cfg(feature = "embassy")]
mod future;
mod instance;
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
pub trait Instance: Peripheral<P = Self> + instance::sealed::Instance + 'static + Send {}

/// DMA peripheral identifier definition.
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

/// DMA channel identifier.
#[derive(PartialEq, Clone, Copy)]
#[derive(Debug, Eq)]
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
    // Create a new configuration struct.
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

    /// Create a memory-to-memory configuration.
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

    /// Create a memory-to-peripheral configuration.
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

    /// Create a peripheral-to-memory configuration.
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

    pub fn split(&mut self) -> [DmaChannel<'_, T, M>; 3] {
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
    /// Create a new DMA channel.
    pub(super) fn new(channel: Channel) -> Self {
        Self {
            _t: PhantomData,
            _mode: PhantomData,
            channel,
        }
    }

    /// Bind the channel to a peripheral.
    pub fn bind(&mut self, map: DmaChannelMap) {
        syscfg::set_dma_channel_map(self.channel, map);
    }

    /// Enable the DMA fast-response mode.
    pub fn en_fast_response(&mut self, en: bool) {
        syscfg::en_dma_channel_fast_response(self.channel, en);
    }

    /// Configure the DMA channel.
    #[inline]
    pub fn config(&mut self, config: Config) {
        T::config(self.channel, config)
    }

    /// Start the DMA transfer.
    #[inline]
    pub fn start(&mut self) {
        T::enable(self.channel, true);
    }

    /// Stop or cancel the DMA transfer.
    #[inline]
    pub fn stop(&mut self) {
        T::enable(self.channel, false);
    }

    /// Return the remaining transfer count.
    pub fn remain(&self) -> u16 {
        T::remain_count(self.channel)
    }
}

impl<'d, T: Instance> DmaChannel<'d, T, Blocking> {
    /// Return whether the transfer has finished.
    pub fn is_finish(&self) -> bool {
        T::event_flag(self.channel, Event::TCIF)
    }

    /// Return whether an error occurred.
    pub fn is_error(&self) -> bool {
        T::event_flag(self.channel, Event::TEIF)
    }

    /// Clear the status flags.
    pub fn clear_flag(&mut self, events: EnumSet<Event>) {
        for e in events {
            T::event_clear(self.channel, e);
        }
    }

    /// Wait for the transfer to complete.
    pub fn wait_complet(&self) -> Result<(), Error> {
        while !T::event_flag(self.channel, Event::TCIF) {
            // Check whether an error occurred.
            if T::event_flag(self.channel, Event::TEIF) {
                T::event_clear(self.channel, Event::TEIF);
                return Err(Error::Others);
            }
        }
        T::event_clear(self.channel, Event::TCIF);
        Ok(())
    }

    /// Wait for the half-transfer to complete.
    pub fn wait_half_complet(&self) -> Result<(), Error> {
        while !T::event_flag(self.channel, Event::HTIF) {
            // Check whether an error occurred.
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
    /// Wait for completion.
    pub async fn wait_complet(&self) -> Result<(), Error> {
        if EventFuture::<T>::new(self.channel, Event::TCIF | Event::TEIF).await != Event::TCIF {
            return Err(Error::Others);
        }

        Ok(())
    }

    /// Wait for half completion.
    pub async fn wait_half_complet(&self) -> Result<(), Error> {
        if EventFuture::<T>::new(self.channel, Event::HTIF | Event::TEIF).await != Event::HTIF {
            return Err(Error::Others);
        }

        Ok(())
    }
}
