use crate::bit::*;
use crate::clock::peripheral::PeripheralInterrupt;
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Line {
    // GPIO 0
    Line0 = 0,
    // GPIO 1
    Line1 = 1,
    // GPIO 2
    Line2 = 2,
    // GPIO 3
    Line3 = 3,
    // GPIO 4
    Line4 = 4,
    // GPIO 5
    Line5 = 5,
    // GPIO6
    Line6 = 6,
    // gpio 7
    Line7 = 7,
    // gpio 8
    Line8 = 8,
    // gpio 9
    Line9 = 9,
    // gpio 10
    Line10 = 10,
    // gpio 11
    Line11 = 11,
    // gpio 12
    Line12 = 12,
    // gpio 13
    Line13 = 13,
    // gpio 14
    Line14 = 14,
    // gpio 15
    Line15 = 15,
    // // PVD
    // Line16 = 16,
    // // COMP 1
    // Line17 = 17,
    // // COMP 2
    // Line18 = 18,
    // // RTC
    // Line19 = 19,
    // // LPTIM
    // Line29 = 29,
}

impl PeripheralInterrupt for Line {
    fn interrupt(&self) -> crate::pac::interrupt {
        match *self {
            Line::Line0 | Line::Line1 => PY32f030xx_pac::interrupt::EXTI0_1,
            Line::Line2 | Line::Line3 => PY32f030xx_pac::interrupt::EXTI2_3,
            _ => PY32f030xx_pac::interrupt::EXTI4_15,
        }
    }
}

impl From<usize> for Line {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Line0,
            1 => Self::Line1,
            2 => Self::Line2,
            3 => Self::Line3,
            4 => Self::Line4,
            5 => Self::Line5,
            6 => Self::Line6,
            7 => Self::Line7,
            8 => Self::Line8,
            9 => Self::Line9,
            10 => Self::Line10,
            11 => Self::Line11,
            12 => Self::Line12,
            13 => Self::Line13,
            14 => Self::Line14,
            15 => Self::Line15,
            _ => unreachable!(),
        }
    }
}

/// 信号边缘检测类型
#[derive(PartialEq)]
pub enum Edge {
    /// 上升沿,
    Rising,
    /// 下降沿
    Falling,
    /// 上升沿和下降沿
    RisingFalling,
}

impl Edge {
    pub fn is_rising(&self) -> bool {
        *self == Self::Rising || *self == Self::RisingFalling
    }

    pub fn is_falling(&self) -> bool {
        *self == Self::Falling || *self == Self::RisingFalling
    }
}

pub struct BitIter(pub u32);

impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.trailing_zeros() {
            32 => None,
            idx => {
                self.0 = bit_mask_idx_clear::<1>(idx as usize, self.0);
                Some(idx)
            }
        }
    }
}
