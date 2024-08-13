//! # ADC 主要特性
//! ## 高性能
//! - 12bit、10bit、8bit 和 6bit 分辨率可配置
//! - ADC 转换时间：1us@12bit（1MHz）
//! - 自校准
//! - 可编程的采样时间
//! - 可编程的数据对齐模式
//! - 支持 DMA
//! ## 低功耗
//! - 为低功耗操作，降低 PCLK 频率，而仍然维持合适的 ADC 性能
//! - 等待模式：防止以低频 PCLK 运行产生溢出
//! ## 模拟输入通道
//! - 10 个外部模拟输入通道：PA[7:0]和 PB[1:0]
//! - 1 个内部 temperature sensor 通道
//! - 1 个内部参考电压通道（VREFINT）
//! ## 转换操作启动可以通过
//! - 软件启动
//! - 可配置极性的硬件启动（TIM1、TIM3 或者 GPIO）
//! ## 转换模式
//! - 单次模式(single mode)：可以转换 1 个单通道或者可以扫描一系列通道
//! - 连续模式(continuous mode)：连续转换被选择的通道
//! - 不连续模式(discontinuous mode)：每次触发，转换被选择的通道 1 次
//! ## 中断产生
//! - 在采样结束
//! - 在转换结束
//! - 在连续转换结束
//! - 模拟看门狗事件
//! - 溢出事件
//! ## 模拟看门狗

mod hal;
mod pins;

// use crate::macro_def::pin_af_for_instance_def;
use embassy_hal_internal::Peripheral;

use crate::clock::peripheral::{PeripheralClockIndex, PeripheralEnable};

pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

#[derive(PartialEq)]
enum Id {
    ADC1,
}

impl PeripheralEnable for Id {
    fn clock(&self, en: bool) {
        match *self {
            Self::ADC1 => PeripheralClockIndex::ADC.enable(en),
        }
    }

    fn reset(&self) {
        match *self {
            Self::ADC1 => PeripheralClockIndex::ADC.reset(),
        }
    }
}

// pin_af_for_instance_def!(ADC_IN0, Instance);
// pin_af_for_instance_def!(ADC_IN1, Instance);
// pin_af_for_instance_def!(ADC_IN2, Instance);
// pin_af_for_instance_def!(ADC_IN3, Instance);
// pin_af_for_instance_def!(ADC_IN4, Instance);
// pin_af_for_instance_def!(ADC_IN5, Instance);
// pin_af_for_instance_def!(ADC_IN6, Instance);
// pin_af_for_instance_def!(ADC_IN7, Instance);
// pin_af_for_instance_def!(ADC_IN8, Instance);
// pin_af_for_instance_def!(ADC_IN9, Instance);
