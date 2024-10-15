// mod future;
mod hal;
mod types;

use crate::clock::peripheral::{
    PeripheralClockIndex, PeripheralIdToClockIndex, PeripheralInterrupt,
};
use crate::macro_def::impl_sealed_peripheral_id;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};
use types::*;

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

impl Default for Config {
    fn default() -> Self {
        Self {
            diretion: Direction::PeriphToMemory,
            prioritie: Priorities::Low,
            mode: RepeatMode::OneTime(0),
            memInc: false,
            periphInc: false,
            periphDataSize: Burst::Single,
            memDataSize: Burst::Single,
            memAddr: 0,
            periphAddr: 0,
        }
    }
}

impl Config {
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
    ) -> Result<Self, Error> {
        if periphDataSize != Burst::Single {
            if periphDataSize == Burst::Double {
                if periphAddr % 2 != 0 {
                    return Err(Error::Address);
                }
            } else {
                if periphAddr % 4 != 0 {
                    return Err(Error::Address);
                }
            }
        };

        Ok(Self {
            diretion,
            prioritie,
            mode,
            memInc,
            periphInc,
            periphDataSize,
            memDataSize,
            memAddr,
            periphAddr,
        })
    }

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
        .unwrap()
    }

    pub fn new_mem2periph(
        src_addr: u32,
        src_inc: bool,
        dst_addr: u32,
        dst_inc: bool,
        priorite: Priorities,
        mode: RepeatMode,
        burst: Burst,
    ) -> Config {
        Self {
            diretion: Direction::MemoryToPeriph,
            prioritie: priorite,
            mode,
            memDataSize: burst,
            periphDataSize: burst,
            memAddr: src_addr,
            periphAddr: dst_addr,
            memInc: src_inc,
            periphInc: dst_inc,
        }
    }

    pub fn new_periph2mem(
        src_addr: u32,
        src_inc: bool,
        dst_addr: u32,
        dst_inc: bool,
        priorite: Priorities,
        mode: RepeatMode,
        burst: Burst,
    ) -> Config {
        Self {
            diretion: Direction::PeriphToMemory,
            prioritie: priorite,
            mode,
            memDataSize: burst,
            periphDataSize: burst,
            memAddr: src_addr,
            periphAddr: dst_addr,
            memInc: src_inc,
            periphInc: dst_inc,
        }
    }
}

pub struct AnyDma<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _mode: PhantomData<M>,
}

impl<'d, T: Instance, M: Mode> Drop for AnyDma<'d, T, M> {
    fn drop(&mut self) {}
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

impl<'d, T: Instance, M: Mode> Drop for DmaChannel<'d, T, M> {
    fn drop(&mut self) {
        T::enable(self.channel, false);
    }
}

impl<'d, T: Instance, M: Mode> DmaChannel<'d, T, M> {
    pub(super) fn new(channel: Channel) -> Self {
        if M::is_async() {
            channel.enable_interrupt();
        }

        Self {
            _t: PhantomData,
            _mode: PhantomData,
            channel,
        }
    }

    // 配置dma通道
    #[inline]
    pub fn config(&mut self, config: Config) {
        T::config(self.channel, config)
    }

    // 开始dma
    #[inline]
    pub fn start(&mut self) {
        T::enable(self.channel, true);
    }

    // 停止或取消dma
    #[inline]
    pub fn stop(&mut self) {
        T::enable(self.channel, false);
    }
}

impl<'d, T: Instance> DmaChannel<'d, T, Blocking> {
    // 等待传输完成
    #[inline]
    pub fn wait(&self) {
        let event = match self.channel {
            Channel::Channel1 => Event::TCIF1,
            Channel::Channel2 => Event::TCIF2,
            Channel::Channel3 => Event::TCIF3,
        };

        while !T::event_flag(event) {}
        T::event_clear(event);
    }
}
