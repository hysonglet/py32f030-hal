pub(crate) mod sealed {
    use super::super::types::*;
    use super::super::*;
    use crate::bit::*;
    use crate::delay::wait_for_true_timeout_block;
    use crate::pac;

    pub(crate) trait Instance {
        #[inline]
        fn block() -> &'static pac::flash::RegisterBlock {
            unsafe { pac::FLASH::PTR.as_ref().unwrap() }
        }

        #[inline]
        fn uuid() -> [u8; 16] {
            const UUID_ADDR: u32 = 0x1fff0e00;
            let uuid: *const [u8; 16] = UUID_ADDR as _;

            unsafe { *uuid }
        }

        #[inline]
        fn busy() -> bool {
            Self::block().sr.read().bsy().bit()
        }

        /// Lock the main flash.
        #[inline]
        fn lock() -> Result<(), Error> {
            let block = Self::block();
            block.cr.modify(|_, w| w.lock().set_bit());

            if !block.cr.read().lock().bit() {
                return Err(Error::Lock);
            }

            Ok(())
        }

        /// Unlock the main flash.
        #[inline]
        fn unlock() -> Result<(), Error> {
            const KEY1: u32 = 0x4567_0123;
            const KEY2: u32 = 0xcdef_89ab;

            let block = Self::block();
            if block.cr.read().lock().bit() {
                wait_for_true_timeout_block(WAIT_TICK_TIMEOUT, || !Self::busy())
                    .map_err(|_| Error::Busy)?;

                Self::block().keyr.write(|w| unsafe { w.bits(KEY1) });
                Self::block().keyr.write(|w| unsafe { w.bits(KEY2) });
                if block.cr.read().lock().bit() {
                    return Err(Error::Unlock);
                }
            }
            Ok(())
        }

        /// Lock the option-byte (OB) flash.
        #[allow(unused)]
        #[inline]
        fn lock_ob() -> Result<(), Error> {
            let block = Self::block();
            block.cr.modify(|_, w| w.optlock().set_bit());

            if !block.cr.read().optlock().bit() {
                return Err(Error::Lock);
            }

            Ok(())
        }

        #[allow(unused)]
        unsafe fn obl_launch() -> Result<(), Error> {
            let block = Self::block();
            if block.cr.read().optlock().bit() {
                return Err(Error::Lock);
            }

            // Reload the option bytes into flash; on success the system reboots.
            block.cr.modify(|_, w| w.obl_launch().set_bit());

            Ok(())
        }

        /// Unlock the option-byte (OB) flash.
        #[allow(unused)]
        #[inline]
        fn unlock_ob() -> Result<(), Error> {
            const KEY1: u32 = 0x0819_2a3b;
            const KEY2: u32 = 0x4c5d_6e7f;

            let block = Self::block();
            if block.cr.read().optlock().bit() {
                wait_for_true_timeout_block(WAIT_TICK_TIMEOUT, || !Self::busy())
                    .map_err(|e| Error::Busy)?;

                Self::block().optkeyr.write(|w| unsafe { w.bits(KEY1) });
                Self::block().optkeyr.write(|w| unsafe { w.bits(KEY2) });
                if block.cr.read().optlock().bit() {
                    return Err(Error::Unlock);
                }
            }
            Ok(())
        }

        /// Enable sector write protection.
        #[allow(unused)]
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

        /// Erase the entire main flash.
        #[inline]
        fn mass_erase() {
            Self::block().cr.modify(|_, w| w.mer().set_bit());
            unsafe {
                core::ptr::write_volatile(FLASH_BASE_ADDR as _, 0x12344321);
            }
        }

        /// Erase a page.
        #[inline]
        fn page_erase(page_addr: u32) {
            assert!((FLASH_BASE_ADDR..FLASH_END_ADDR).contains(&page_addr));
            assert!(page_addr.is_multiple_of(FLASH_PAGE_SIZE as u32));
            Self::block().cr.modify(|_, w| w.per().set_bit());
            unsafe {
                core::ptr::write_volatile(page_addr as _, 0xff);
            }
        }

        /// Erase a sector.
        #[inline]
        fn sector_erase(sector_addr: u32) {
            assert!((FLASH_BASE_ADDR..FLASH_END_ADDR).contains(&sector_addr));
            assert!(sector_addr.is_multiple_of(FLASH_SECTOR_SIZE as u32));

            Self::block().cr.modify(|_, w| w.ser().set_bit());
            unsafe {
                core::ptr::write_volatile(sector_addr as _, 0xff);
            }
        }

        /// Program a page.
        #[inline]
        fn page_program(page_addr: u32, content: [u32; FLASH_PAGE_SIZE / 4]) {
            assert!((FLASH_BASE_ADDR..FLASH_END_ADDR).contains(&page_addr));
            assert!(page_addr.is_multiple_of(FLASH_PAGE_SIZE as u32));

            let block = Self::block();

            // Start programming.
            block.cr.modify(|_, w| w.pg().set_bit());

            // Avoid interruption.
            critical_section::with(|_cs| {
                content.iter().enumerate().for_each(|(i, v)| unsafe {
                    core::ptr::write_volatile((page_addr + i as u32 * 4) as _, *v);
                    if i == 30 {
                        block.cr.modify(|_, w| w.pgtstrt().set_bit());
                    }
                });
            });

            // Wait for programming to finish.
            while Self::busy() {}

            // Disable programming.
            block.cr.modify(|_, w| w.pg().clear_bit());
        }

        /// Set the Flash read wait states (ACR.LATENCY): 0–1 wait states.
        #[inline]
        fn set_latency(ws: u8) {
            Self::block()
                .acr
                .modify(|_, w| w.latency().bit(ws.min(1) != 0));
        }

        /// Read the current Flash read wait states.
        #[inline]
        fn get_latency() -> u8 {
            Self::block().acr.read().latency().bit() as u8
        }

        /// Read the raw option byte register (OPTR) value.
        #[inline]
        fn read_option_byte() -> u32 {
            Self::block().optr.read().bits()
        }

        /// Read the current read-protection level (OPTR.RDP).
        #[inline]
        fn get_read_protection() -> u8 {
            Self::block().optr.read().rdp().bits()
        }
    }
}
