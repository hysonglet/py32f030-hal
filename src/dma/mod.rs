use core::marker::PhantomData;

use crate::clock::peripheral::PeripheralClock;
use critical_section::CriticalSection;
use embassy_hal_internal::{into_ref, Peripheral};

mod hal;

pub enum Priorities {
    Low = 0,
    Medium = 1,
    High = 2,
    VeryHigh = 3,
}

#[derive(Clone, Copy)]
pub enum Burst {
    // 1 byte
    Single = 0,
    // 2 bytes
    Double = 1,
    // 4 bytes
    World = 2,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Channel {
    Channel1 = 1,
    Channel2 = 2,
    Channel3 = 3,
}

// impl From<usize> for Channel {
//     fn from(value: usize) -> Channel {
//         match value {
//             1 => Self::Channel1,
//             2 => Self::Channel2,
//             3 => Self::Channel3,
//             _ => unreachable!(),
//         }
//     }
// }

#[derive(PartialEq)]
pub enum Mode {
    OneTime(u16),
    Repeat(u16),
}

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

pub struct FlexDmaChannel<T: DmaChannel> {
    _p: PhantomData<T>,
}

impl<T: DmaChannel> FlexDmaChannel<T> {
    pub fn config(config: Config) -> Result<(), Error> {
        T::config(config)
    }

    pub fn new(_dmachannel: impl Peripheral<P = T>, config: Config) -> Result<Self, Error> {
        into_ref!(_dmachannel);
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

impl<T: DmaChannel> Drop for FlexDmaChannel<T> {
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
        let mut config = Self::default();

        config.diretion = Direction::MemoryToMemory;
        config.prioritie = priorite;

        config.mode = mode;
        config.memDataSize = burst;
        config.periphDataSize = burst;

        config.memAddr = src_addr;
        config.periphAddr = dst_addr;
        config.memInc = src_inc;
        config.periphInc = dst_inc;

        config
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
        let mut config = Self::default();

        config.diretion = Direction::MemoryToPeriph;
        config.prioritie = priorite;
        config.mode = mode;
        config.memDataSize = burst;
        config.periphDataSize = burst;

        config.memAddr = src_addr;
        config.periphAddr = dst_addr;
        config.memInc = src_inc;
        config.periphInc = dst_inc;

        config
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
        let mut config = Self::default();

        config.diretion = Direction::PeriphToMemory;
        config.prioritie = priorite;
        config.mode = mode;
        config.memDataSize = burst;
        config.periphDataSize = burst;

        config.memAddr = src_addr;
        config.periphAddr = dst_addr;
        config.memInc = src_inc;
        config.periphInc = dst_inc;

        config
    }
}

struct Dma;

impl Dma {
    fn enable(en: bool) {
        PeripheralClock::DMA.enable(en);
    }

    fn reset() {
        PeripheralClock::DMA.reset();
    }

    fn init() {
        Self::reset();
        Self::enable(true);
    }
}

pub fn init(_cs: CriticalSection) {
    Dma::init();
}
