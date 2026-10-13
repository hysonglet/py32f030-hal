//! Power management (PWR)
//!
//! Provides: RTC/backup-domain access enable, programmable voltage detector (PVD),
//! core voltage scaling (VOS), low-power run (LPR), SRAM retention configuration,
//! and entry points for the Sleep / Stop low-power modes.

use crate::pac;

mod types;

pub use types::*;

/// Address of the SCB system control register (SCR); SLEEPDEEP is bit 2.
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

pub struct pwr;

impl pwr {
    fn block() -> &'static pac::pwr::RegisterBlock {
        unsafe { pac::PWR::PTR.as_ref().unwrap() }
    }

    /// Power control register block (shared with RTC and other modules).
    pub(crate) fn rtc_unlock(unlock: bool) {
        Self::block().cr1.modify(|_, w| w.dbp().bit(unlock));
    }

    /// Backup domain write protection (DBP).
    ///
    /// After reset it is 0, so RTC and backup registers are read-only; set it to 1
    /// before accessing them.
    pub fn set_backup_write_access(en: bool) {
        Self::block().cr1.modify(|_, w| w.dbp().bit(en));
    }

    /// Set the core voltage scaling level.
    ///
    /// Note: CR1.VOS is a single bit, so it only distinguishes Range1 (bit=0) from
    /// the other low-power levels (bit=1).
    pub fn set_voltage_scale(scale: VoltageScale) {
        Self::block()
            .cr1
            .modify(|_, w| w.vos().bit(scale as u8 != 0));
    }

    /// Low-power run mode (CR1.LPR).
    ///
    /// Once set, the core enters low-power run; combine with
    /// `set_voltage_scale(Range3)` to reduce power further.
    pub fn set_low_power_run(en: bool) {
        Self::block().cr1.modify(|_, w| w.lpr().bit(en));
    }

    /// SRAM retention configuration (CR1.SRAM_RETv): whether SRAM contents are kept
    /// in Stop/Standby.
    pub fn set_sram_retention(en: bool) {
        Self::block()
            .cr1
            .modify(|_, w| unsafe { w.sram_retv().bits(en as u8) });
    }

    /// Keep the HSI oscillator running in Stop/Standby (CR1.HSION_CTRL).
    pub fn set_hsi_on_in_stop(en: bool) {
        Self::block().cr1.modify(|_, w| w.hsion_ctrl().bit(en));
    }

    /// Configure and (optionally) enable the PVD.
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

    /// Current PVD output: true means VDD is above the threshold, false means it has
    /// dropped below.
    #[inline]
    pub fn pvd_output() -> bool {
        Self::block().sr.read().pvdo().bit()
    }

    /// Enter Sleep mode.
    ///
    /// Only the CPU clock is stopped; peripherals and the core keep running and any
    /// interrupt or event can wake the system.
    #[inline]
    pub fn enter_sleep() {
        // Sleep mode: SLEEPDEEP = 0.
        set_sleepdeep(false);
        cortex_m::asm::wfi();
    }

    /// Enter Stop mode.
    ///
    /// The high-speed clock is stopped and the core halts. A wakeup source (RTC
    /// alarm, external interrupt, etc.) must be enabled first; after wakeup,
    /// execution resumes from the instruction following `enter_stop` and
    /// `wake_from_low_power()` should be called.
    #[inline]
    pub fn enter_stop() {
        // Stop mode: SLEEPDEEP = 1.
        set_sleepdeep(true);
        cortex_m::asm::wfi();
        // Clear deep sleep after wakeup to avoid re-entering it.
        Self::wake_from_low_power();
    }

    /// Call after exiting low-power mode: clears the SLEEPDEEP bit.
    #[inline]
    pub fn wake_from_low_power() {
        set_sleepdeep(false);
    }
}
