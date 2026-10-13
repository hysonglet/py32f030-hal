pub mod sealed {
    use super::super::types::*;
    use crate::bit::*;
    use crate::exti::Line;
    use crate::pac;

    #[allow(dead_code)]
    pub(crate) trait Instance {
        #[inline]
        fn block() -> &'static pac::exti::RegisterBlock {
            unsafe { pac::EXTI::PTR.as_ref().unwrap() }
        }

        #[inline]
        fn line_ring_edge(line: Line, en: bool) {
            Self::block().rtsr.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(line as usize, r.bits(), en as u32))
            })
        }

        #[inline]
        fn line_falling_edge(line: Line, en: bool) {
            Self::block().ftsr.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(line as usize, r.bits(), en as u32))
            })
        }

        // #[inline]
        // fn get_pending(line: Line) -> bool {
        //     bit_mask_idx_get::<1>(line as usize, Self::block().pr.read().bits()) != 0
        // }
        #[inline]
        fn clear_pending(line: Line) {
            Self::block()
                .pr
                .modify(|r, w| unsafe { w.bits(bit_mask_idx_set::<1>(line as usize, r.bits())) })
        }

        /// Read the raw pending bit (no distinction between GPIO / internal lines)
        #[inline]
        fn pending_raw(line: Line) -> bool {
            bit_mask_idx_get::<1>(line as usize, Self::block().pr.read().bits()) != 0
        }

        /// Software-trigger an EXTI line (SWIER)
        #[inline]
        fn software_trigger(line: Line) {
            Self::block()
                .swier
                .modify(|r, w| unsafe { w.bits(bit_mask_idx_set::<1>(line as usize, r.bits())) })
        }

        #[inline]
        fn exit_channle_select(line: Line, pin: ExitPinSource) {
            let block = Self::block();
            let idx = line as usize;
            // Each pin source occupies 2 bits; bit offset = (line % 4) * 8
            let shift = (idx % 4) * 8;
            let val = pin as u32;
            match idx / 4 {
                0 => block.exticr1.modify(|r, w| unsafe {
                    w.bits(bit_mask_idx_modify::<2>(shift, r.bits(), val))
                }),
                1 => block.exticr2.modify(|r, w| unsafe {
                    w.bits(bit_mask_idx_modify::<2>(shift, r.bits(), val))
                }),
                2 => block.exticr3.modify(|r, w| unsafe {
                    w.bits(bit_mask_idx_modify::<2>(shift, r.bits(), val))
                }),
                // line 12–15: PY32f030xx-pac does not provide exticr4, so the source
                // cannot be switched (PA is the default; PB/PF are unavailable on these
                // lines), so skip.
                _ => {}
            }
        }

        #[inline]
        fn line_pend_enable(line: Line, en: bool) {
            Self::block().imr.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(line as usize, r.bits(), en as u32))
            });
        }

        #[inline]
        fn is_line_pend_enable(line: Line) -> bool {
            bit_mask_idx_get::<1>(line as usize, Self::block().imr.read().bits()) != 0
        }

        // fn line_event_wakeup_enable(line: Line, en: bool) {
        //     Self::block().emr.modify(|r, w| unsafe {
        //         w.bits(bit_mask_idx_modify::<1>(
        //             line as usize,
        //             r.bits(),
        //             en as u32,
        //         ))
        //     })
        // }
    }

    /// Internal register-access marker type (used by the `Exti` public API and `ExtiInputFuture`)
    pub(crate) struct ExtiImpl;
    impl Instance for ExtiImpl {}
}
