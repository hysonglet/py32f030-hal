use super::Rcc;
use crate::common::{BitOption, Peripheral};

#[derive(Clone, Copy)]
pub enum GpioClock {
    GPIOA = 0,
    GPIOB = 1,
    GPIOF = 5,
}

impl GpioClock {
    #[inline]
    pub fn enable(&self, en: bool) {
        Rcc::peripheral().iopenr.modify(|r, w| unsafe {
            w.bits(BitOption::bit_mask_idx_modify::<1>(
                *self as usize,
                r.bits(),
                en as u32,
            ))
        })
    }
    #[inline]
    pub fn reset(&self) {
        Rcc::peripheral().ioprstr.modify(|r, w| unsafe {
            w.bits(BitOption::bit_mask_idx_set::<1>(*self as usize, r.bits()))
        })
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum PeripheralClock {
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
}

impl PeripheralClock {
    pub fn enable(&self, en: bool) {
        if (*self as u32) < 32 {
            Rcc::peripheral().ahbenr.modify(|r, w| unsafe {
                w.bits(BitOption::bit_mask_idx_modify::<1>(
                    *self as usize,
                    r.bits(),
                    en as u32,
                ))
            })
        } else if (*self as u32) < 64 {
            Rcc::peripheral().apbenr1.modify(|r, w| unsafe {
                w.bits(BitOption::bit_mask_idx_modify::<1>(
                    (*self as usize) - 32,
                    r.bits(),
                    en as u32,
                ))
            })
        } else {
            Rcc::peripheral().apbenr2.modify(|r, w| unsafe {
                w.bits(BitOption::bit_mask_idx_modify::<1>(
                    (*self as usize) - 64,
                    r.bits(),
                    en as u32,
                ))
            })
        }
    }

    pub fn reset(&self) {
        if (*self as u32) < 32 {
            if *self == Self::FLASH || *self == Self::SRAM {
                panic!()
            }
            Rcc::peripheral().ahbrstr.modify(|r, w| unsafe {
                w.bits(BitOption::bit_mask_idx_set::<1>(*self as usize, r.bits()))
            })
        } else if (*self as u32) < 64 {
            Rcc::peripheral().apbrstr1.modify(|r, w| unsafe {
                w.bits(BitOption::bit_mask_idx_set::<1>(
                    (*self as usize) - 32,
                    r.bits(),
                ))
            })
        } else {
            Rcc::peripheral().apbrstr2.modify(|r, w| unsafe {
                w.bits(BitOption::bit_mask_idx_set::<1>(
                    (*self as usize) - 64,
                    r.bits(),
                ))
            })
        }
    }
}