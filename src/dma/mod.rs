use core::marker::PhantomData;

use critical_section::CriticalSection;
use embassy_hal_internal::{into_ref, Peripheral};

use crate::clock::peripheral::PeripheralClockIndex;

mod hal;

/// 传输的优先级
pub enum Priorities {
    Low = 0,
    Medium = 1,
    High = 2,
    VeryHigh = 3,
}

/// DMA 传输的宽度
#[derive(Clone, Copy)]
pub enum Burst {
    // 1 byte
    Single = 0,
    // 2 bytes
    Double = 1,
    // 4 bytes
    World = 2,
}

/// 通道 id
#[derive(PartialEq, Clone, Copy)]
pub enum Channel {
    Channel1 = 1,
    Channel2 = 2,
    Channel3 = 3,
}

/// DMA模式，单次或循环
#[derive(PartialEq)]
pub enum Mode {
    OneTime(u16),
    Repeat(u16),
}

/// DMA传输方向
#[derive(PartialEq)]
pub enum Direction {
    PeriphToMemory,
    MemoryToPeriph,
    MemoryToMemory,
}

pub trait DmaChannel: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

/// 为 dma channle对象 impl 接口
macro_rules! impl_sealed_dma_channel {
    (
        $peripheral: ident, $dma_channel: ident
    ) => {
        impl hal::sealed::Instance for crate::mcu::peripherals::$peripheral {
            fn channel() -> Channel {
                Channel::$dma_channel
            }
        }

        impl DmaChannel for crate::mcu::peripherals::$peripheral {}
    };
}

impl_sealed_dma_channel!(DmaChannel1, Channel1);
impl_sealed_dma_channel!(DmaChannel2, Channel2);
impl_sealed_dma_channel!(DmaChannel3, Channel3);

pub struct AnyChannel<T: DmaChannel> {
    _p: PhantomData<T>,
}

impl<T: DmaChannel> AnyChannel<T> {
    pub fn config(config: Config) -> Result<(), Error> {
        T::config(config)
    }

    pub fn new(_dmachannel: impl Peripheral<P = T>, config: Config) -> Result<Self, Error> {
        into_ref!(_dmachannel);

        // 关闭通道，dma 通道配置只有在 en 为 0 时候才能有效配置
        T::enable(false);

        T::config(config)?;

        Ok(Self { _p: PhantomData })
    }

    pub fn start(&self) {
        T::enable(true);
    }
    pub fn stop(&self) {
        T::enable(false);
    }

    pub fn wait_finish_block(&self) {
        // 剩余传输数量
        while T::remain_count() != 0 {}
    }
}

impl<T: DmaChannel> Drop for AnyChannel<T> {
    fn drop(&mut self) {
        T::enable(false);
    }
}

#[derive(Debug)]
pub enum Error {
    Busy,
}

pub struct Config {
    diretion: Direction,
    prioritie: Priorities,
    mode: Mode,
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
            mode: Mode::OneTime(0),
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
    pub fn new_mem2mem(
        src_addr: u32,
        src_inc: bool,
        dst_addr: u32,
        dst_inc: bool,
        priorite: Priorities,
        mode: Mode,
        burst: Burst,
    ) -> Self {
        Self {
            diretion: Direction::MemoryToMemory,
            prioritie: priorite,
            mode,
            memDataSize: burst,
            periphDataSize: burst,
            memAddr: src_addr,
            memInc: src_inc,
            periphAddr: dst_addr,
            periphInc: dst_inc,
        }
    }

    pub fn new_mem2periph(
        src_addr: u32,
        src_inc: bool,
        dst_addr: u32,
        dst_inc: bool,
        priorite: Priorities,
        mode: Mode,
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
        mode: Mode,
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

struct Dma;

impl Dma {
    #[inline]
    fn enable(en: bool) {
        PeripheralClockIndex::DMA.clock(en);
    }

    #[inline]
    fn reset() {
        PeripheralClockIndex::DMA.reset();
    }

    #[inline]
    fn init() {
        Self::enable(true);
        Self::reset();
    }
}

pub fn init(_cs: CriticalSection) {
    Dma::init();
}
