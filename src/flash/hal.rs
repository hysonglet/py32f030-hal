pub(crate) mod sealed {
    use super::super::types::*;
    use super::super::*;
    use crate::bit::*;
    use crate::pac;

    pub(crate) trait Instance {
        #[inline]
        fn block() -> &'static pac::flash::RegisterBlock {
            unsafe { pac::FLASH::PTR.as_ref().unwrap() }
        }

        #[inline]
        fn uuid() -> [u8; 16] {
            const UUID_ADDR: u32 = 0x1fff0e00;;
            let uuid: *const [u8; 16] = UUID_ADDR as _;

            unsafe { *uuid }
        }

        /// 锁定 main flash
        #[inline]
        fn lock() -> Result<(), Error> {
            let block = Self::block();
            block.cr.modify(|_, w| w.lock().set_bit());

            if !block.cr.read().lock().bit() {
                return Err(Error::Lock);
            }

            Ok(())
        }

        /// 解锁 main flash
        #[inline]
        fn unlock() -> Result<(), Error> {
            const KEY1: u32 = 0x4567_0123;
            const KEY2: u32 = 0xcdef_89ab;

            let block = Self::block();
            if block.cr.read().lock().bit() {
                if block.sr.read().bsy().bit() {
                    return Err(Error::Busy);
                }
                Self::block().keyr.write(|w| unsafe { w.bits(KEY1) });
                Self::block().keyr.write(|w| unsafe { w.bits(KEY2) });
                if block.cr.read().lock().bit() {
                    return Err(Error::Unlock);
                }
            }
            Ok(())
        }

        /// 锁定 ob flash
        #[inline]
        fn lock_ob() -> Result<(), Error> {
            let block = Self::block();
            block.cr.modify(|_, w| w.optlock().set_bit());

            if !block.cr.read().optlock().bit() {
                return Err(Error::Lock);
            }

            Ok(())
        }

        unsafe fn obl_launch() -> Result<(), Error> {
            let block = Self::block();
            if block.cr.read().optlock().bit() {
                return Err(Error::Lock);
            }

            // 重载 op 到 flash， 成功系统将会重启
            block.cr.modify(|_, w| w.obl_launch().set_bit());

            Ok(())
        }

        /// 解锁 ob flash
        #[inline]
        fn unlock_ob() -> Result<(), Error> {
            const KEY1: u32 = 0x0819_2a3b;
            const KEY2: u32 = 0x4c5d_6e7f;

            let block = Self::block();
            if block.cr.read().optlock().bit() {
                if block.sr.read().bsy().bit() {
                    return Err(Error::Busy);
                }
                Self::block().optkeyr.write(|w| unsafe { w.bits(KEY1) });
                Self::block().optkeyr.write(|w| unsafe { w.bits(KEY2) });
                if block.cr.read().optlock().bit() {
                    return Err(Error::Unlock);
                }
            }
            Ok(())
        }

        /// 使能扇区写保护
        #[inline]
        fn en_sector_protect(sector: usize, en: bool) {
            assert!(sector < FLASH_SECTOR_CNT);
            Self::block().wrpr.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(
                    sector,
                    r.bits(),
                    if en { 0 } else { 1 },
                ))
            });
        }

        fn mass_erase() {
            Self::block().cr.modify(|_, w| w.mer().set_bit());
        }
    }
}
