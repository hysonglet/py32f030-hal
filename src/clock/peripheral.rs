use super::Rcc;
use crate::bit::*;

#[derive(PartialEq, Clone, Copy)]
pub enum PeripheralClockIndex {
    DMA = 0,
    FLASH = 8,
    SRAM = 9,
    CRC = 12,

    TIM3 = 32 + 1,
    RTCAPB = 32 + 10,
    WWDG = 32 + 11,
    SPI2 = 32 + 14,
    UART2 = 32 + 17,
    I2C = 21 + 32,
    DBG = 27 + 32,
    PWR = 28 + 32,
    LPTIM = 31 + 32,

    SYSCFG = 64,
    TIM1 = 11 + 64,
    SPI1 = 12 + 64,
    USART1 = 14 + 64,
    TIM14 = 15 + 64,
    TIM16 = 17 + 64,
    TIM17 = 18 + 64,
    ADC = 20 + 64,
    COMP1 = 21 + 64,
    COMP2 = 22 + 64,
    LED = 23 + 64,

    GPIOA = 96,
    GPIOB = 1 + 96,
    GPIOF = 5 + 96,
}

impl PeripheralClockIndex {
    /// 返回时钟开启状态
    pub fn is_open(&self) -> bool {
        let idx = *self as usize;
        if idx < 32 {
            bit_mask_idx_get::<1>(idx, Rcc::block().ahbenr.read().bits()) != 0
        } else if idx < 64 {
            bit_mask_idx_get::<1>(idx - 32, Rcc::block().apbenr1.read().bits()) != 0
        } else if idx < 96 {
            bit_mask_idx_get::<1>(idx - 64, Rcc::block().apbenr2.read().bits()) != 0
        } else {
            bit_mask_idx_get::<1>(idx - 96, Rcc::block().iopenr.read().bits()) != 0
        }
    }

    /// 设置时钟开启或关闭
    pub fn clock(&self, en: bool) {
        let idx = *self as usize;
        if idx < 32 {
            Rcc::block().ahbenr.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(idx, r.bits(), en as u32))
            })
        } else if idx < 64 {
            Rcc::block().apbenr1.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(idx - 32, r.bits(), en as u32))
            })
        } else if idx < 96 {
            Rcc::block().apbenr2.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(idx - 64, r.bits(), en as u32))
            })
        } else {
            Rcc::block().iopenr.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(idx - 96, r.bits(), en as u32))
            })
        }
    }

    /// 关闭外设时钟
    #[inline]
    pub fn close(&self) {
        self.clock(false);
    }

    /// 开启外设时钟
    #[inline]
    pub fn open(&self) {
        self.clock(true);
    }

    /// 复位外设
    pub fn reset(&self) {
        let idx = *self as usize;
        if idx < 32 {
            if *self == Self::FLASH || *self == Self::SRAM {
                panic!()
            }
            Rcc::block()
                .ahbrstr
                .modify(|r, w| unsafe { w.bits(bit_mask_idx_set::<1>(idx, r.bits())) })
        } else if idx < 64 {
            Rcc::block()
                .apbrstr1
                .modify(|r, w| unsafe { w.bits(bit_mask_idx_set::<1>(idx - 32, r.bits())) })
        } else if idx < 96 {
            Rcc::block()
                .apbrstr2
                .modify(|r, w| unsafe { w.bits(bit_mask_idx_set::<1>(idx - 64, r.bits())) })
        } else {
            Rcc::block()
                .ioprstr
                .modify(|r, w| unsafe { w.bits(bit_mask_idx_set::<1>(idx - 96, r.bits())) })
        }
    }
}

pub trait PeripheralInterrupt {
    fn interrupt(&self) -> crate::pac::interrupt;

    #[inline]
    fn enable_interrupt(&self) {
        unsafe { cortex_m::peripheral::NVIC::unmask(self.interrupt()) }
    }

    #[inline]
    fn disable_interrupt(&self) {
        cortex_m::peripheral::NVIC::mask(self.interrupt())
    }
}

pub trait PeripheralIdToClockIndex {
    fn clock(&self) -> PeripheralClockIndex;
}
