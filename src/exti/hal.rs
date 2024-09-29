pub mod sealed {
    use crate::bit::*;
    use crate::exti::Line;
    use crate::gpio::GpioPort;
    use crate::{gpio, pac};

    #[derive(Debug, PartialEq)]
    pub enum ExitPinSource {
        PA = 0,
        PB = 1,
        PF = 2,
    }

    impl From<gpio::GpioPort> for ExitPinSource {
        fn from(value: GpioPort) -> Self {
            match value {
                GpioPort::GPIOA => Self::PA,
                GpioPort::GPIOB => Self::PB,
                GpioPort::GPIOF => Self::PF,
            }
        }
    }

    pub(crate) trait Instance {
        #[inline]
        fn block() -> &'static pac::exti::RegisterBlock {
            unsafe { pac::EXTI::PTR.as_ref().unwrap() }
        }

        #[inline]
        fn line_ring_edge(line: Line, en: bool) {
            Self::block().rtsr.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(
                    line as usize * 1,
                    r.bits(),
                    en as u32,
                ))
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

        #[inline]
        fn exit_channle_select(line: Line, pin: ExitPinSource) {
            let block = Self::block();
            // let pin = pin as u32;
            match line {
                Line::Line0 | Line::Line1 | Line::Line2 | Line::Line3 => {
                    block.exticr1.modify(|r, w| unsafe {
                        w.bits(bit_mask_idx_modify::<2>(
                            line as usize * 8,
                            r.bits(),
                            pin as u32,
                        ))
                    })
                }
                Line::Line4 => block.exticr2.modify(|r, w| unsafe {
                    w.bits(bit_mask_idx_modify::<2>(0, r.bits(), pin as u32))
                }),
                Line::Line5 | Line::Line6 | Line::Line7 => block.exticr2.modify(|r, w| unsafe {
                    assert!(pin != ExitPinSource::PF);
                    w.bits(bit_mask_idx_modify::<1>(
                        (line as usize - 4) * 8,
                        r.bits(),
                        pin as u32,
                    ))
                }),
                Line::Line8 => block.exticr3.modify(|r, w| unsafe {
                    assert!(pin == ExitPinSource::PF);
                    w.bits(bit_mask_idx_modify::<1>(0, r.bits(), pin as u32))
                }),
                Line::Line9
                | Line::Line10
                | Line::Line11
                | Line::Line12
                | Line::Line13
                | Line::Line14
                | Line::Line15 => {
                    assert!(pin != ExitPinSource::PB);
                    assert!(pin != ExitPinSource::PF);
                } // _ => {
                  //     panic!()
                  // }
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
}
