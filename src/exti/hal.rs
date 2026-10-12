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

        /// 读取原始挂起位（不区分 GPIO / 内部线）
        #[inline]
        fn pending_raw(line: Line) -> bool {
            bit_mask_idx_get::<1>(line as usize, Self::block().pr.read().bits()) != 0
        }

        /// 软件触发某一 EXTI 线（SWIER）
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
            // 每个引脚源占 2 位，位偏移 = (line % 4) * 8
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
                // line 12–15：PY32f030xx-pac 未提供 exticr4，无法切换源选择
                // （PA 为默认值，PB/PF 在这些线上不可用），故跳过。
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

    /// 内部寄存器访问标记类型（供 `Exti` 公共 API 与 `ExtiInputFuture` 使用）
    pub(crate) struct ExtiImpl;
    impl Instance for ExtiImpl {}
}
