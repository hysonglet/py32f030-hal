use crate::pac;
pub(crate) struct pwr;

impl pwr {
    fn block() -> &'static pac::pwr::RegisterBlock {
        unsafe { pac::PWR::PTR.as_ref().unwrap() }
    }

    /// RTC 写保护禁止,在复位后， RTC 处于写保护状态以防意外写入。要访问 RTC 该位必须设置为 1。
    /// 0：禁止访问 RTC
    /// 1：可以访问 RTC
    pub fn rtc_unlock(unlock: bool) {
        Self::block().cr1.modify(|_, w| w.dbp().bit(unlock));
    }
}
