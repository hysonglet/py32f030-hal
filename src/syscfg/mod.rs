mod types;

use crate::pac;
pub use types::*;

use crate::bit::*;
use crate::dma::Channel;

pub struct syscfg;

impl syscfg {
    fn block() -> &'static pac::syscfg::RegisterBlock {
        unsafe { pac::SYSCFG::ptr().as_ref().unwrap() }
    }

    fn clock() -> crate::clock::peripheral::PeripheralClockIndex {
        crate::clock::peripheral::PeripheralClockIndex::SYSCFG
    }

    pub fn open() {
        Self::clock().open();
    }

    pub fn close() {
        Self::clock().close();
    }

    /// 设置dma通道映射
    pub fn set_dma_channel_map(channel: Channel, map: DmaChannelMap) {
        Self::block().cfgr3.modify(|r, w| unsafe {
            w.bits(bit_mask_idx_modify::<5>(
                channel as usize * 8,
                r.bits(),
                map as u32,
            ))
        });
    }

    /// 使能dma快速响应
    pub fn en_dma_channel_fast_response(channel: Channel, en: bool) {
        Self::block().cfgr3.modify(|r, w| unsafe {
            w.bits(bit_mask_idx_modify::<1>(
                channel as usize * 8 + 5,
                r.bits(),
                if en { 1 } else { 0 },
            ))
        });
    }

    /// 设置系统启动引导的地址
    pub fn set_boot_mode(mode: BootMode) {
        Self::block()
            .cfgr1
            .modify(|_, w| unsafe { w.mem_mode().bits(mode as u8) });
    }
}
