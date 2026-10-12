//! 电源管理（PWR）
//!
//! 提供：RTC/备份域访问使能、可编程电压检测器（PVD）、内核电压缩放（VOS）、
//! 低功耗运行（LPR）、SRAM 保持配置，以及进入 Sleep / Stop 低功耗模式的入口。

use crate::pac;

/// SCB 系统控制寄存器（SCR）地址，SLEEPDEEP 为 bit 2
const SCB_SCR: *mut u32 = 0xE000_ED10 as *mut u32;

#[inline]
fn set_sleepdeep(deep: bool) {
    unsafe {
        let v = core::ptr::read_volatile(SCB_SCR);
        if deep {
            core::ptr::write_volatile(SCB_SCR, v | (1 << 2));
        } else {
            core::ptr::write_volatile(SCB_SCR, v & !(1 << 2));
        }
    }
}

/// 内核电压缩放等级（CR1.VOS）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VoltageScale {
    /// 等级 1（最高性能，最高功耗）
    Range1 = 0,
    /// 等级 2（低功耗）
    Range2 = 1,
    /// 等级 3（最低功耗）
    Range3 = 2,
}

/// 可编程电压检测器（PVD）配置
///
/// 阈值由 3 位值选择（0–7），对应 1.9V–3.3V 区间，详见参考手册 PVD 阈值表。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PvdConfig {
    /// 是否使能 PVD
    pub enable: bool,
    /// 阈值等级（0–7）
    pub threshold: u8,
    /// PVD 上升沿/下降沿过滤使能
    pub filter: bool,
    /// 过滤器时间（CR2.FLT_TIME，0–7）
    pub filter_time: u8,
    /// 当 PVD 输出低于阈值时是否触发中断
    pub interrupt: bool,
}

pub(crate) struct pwr;

impl pwr {
    fn block() -> &'static pac::pwr::RegisterBlock {
        unsafe { pac::PWR::PTR.as_ref().unwrap() }
    }

    /// 电源控制寄存器块（供 RTC 等模块复用）
    pub(crate) fn rtc_unlock(unlock: bool) {
        Self::block().cr1.modify(|_, w| w.dbp().bit(unlock));
    }

    /// 备份域写保护（DBP）
    ///
    /// 复位后为 0，此时 RTC 与备份寄存器不可写；访问前需置 1。
    pub fn set_backup_write_access(en: bool) {
        Self::block().cr1.modify(|_, w| w.dbp().bit(en));
    }

    /// 设置内核电压缩放等级
    ///
    /// 注意：CR1.VOS 为单 bit，故仅区分 Range1（bit=0）与其余低功耗档位（bit=1）。
    pub fn set_voltage_scale(scale: VoltageScale) {
        Self::block()
            .cr1
            .modify(|_, w| w.vos().bit(scale as u8 != 0));
    }

    /// 低功耗运行模式（CR1.LPR）
    ///
    /// 置位后内核进入低功耗运行，配合 `set_voltage_scale(Range3)` 可进一步降低功耗。
    pub fn set_low_power_run(en: bool) {
        Self::block().cr1.modify(|_, w| w.lpr().bit(en));
    }

    /// SRAM 保持配置（CR1.SRAM_RETv）：停止/待机时是否保留 SRAM 内容
    pub fn set_sram_retention(en: bool) {
        Self::block()
            .cr1
            .modify(|_, w| unsafe { w.sram_retv().bits(en as u8) });
    }

    /// 停止/待机时保持 HSI 振荡器运行（CR1.HSION_CTRL）
    pub fn set_hsi_on_in_stop(en: bool) {
        Self::block().cr1.modify(|_, w| w.hsion_ctrl().bit(en));
    }

    /// 配置并（可选）使能 PVD
    pub fn config_pvd(config: PvdConfig) {
        Self::block().cr2.modify(|_, w| unsafe {
            w.pvde()
                .bit(config.enable)
                .pvdt()
                .bits(config.threshold.min(7))
                .flten()
                .bit(config.filter)
                .flt_time()
                .bits(config.filter_time.min(7))
        });
    }

    /// PVD 当前输出：true 表示 VDD 高于阈值，false 表示已跌破阈值
    #[inline]
    pub fn pvd_output() -> bool {
        Self::block().sr.read().pvdo().bit()
    }

    /// 进入 Sleep（睡眠）模式
    ///
    /// 仅关闭 CPU 时钟，外设与内核继续运行；任意中断或事件均可唤醒。
    #[inline]
    pub fn enter_sleep() {
        // Sleep 模式：SLEEPDEEP = 0
        set_sleepdeep(false);
        cortex_m::asm::wfi();
    }

    /// 进入 Stop（停机）模式
    ///
    /// 关闭高速时钟，内核停摆；需先使能某个唤醒源（RTC 闹钟、外部中断等），
    /// 唤醒后从 `enter_stop` 之后的指令继续执行，并应调用 `wake_from_low_power()`。
    #[inline]
    pub fn enter_stop() {
        // Stop 模式：SLEEPDEEP = 1
        set_sleepdeep(true);
        cortex_m::asm::wfi();
        // 唤醒后退出深度睡眠，避免再次进入
        Self::wake_from_low_power();
    }

    /// 退出低功耗模式后调用：清除 SLEEPDEEP 位
    #[inline]
    pub fn wake_from_low_power() {
        set_sleepdeep(false);
    }
}
