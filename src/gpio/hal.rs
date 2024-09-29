pub(crate) mod sealed {
    use super::super::*;
    use crate::bit::*;
    use crate::pac;

    pub trait Pin {
        fn port_pin(&self) -> u8;

        #[inline]
        fn port(&self) -> GpioPort {
            let port = (self.port_pin() >> 4) as usize;
            assert!(port < 3);
            port.into()
        }

        #[inline]
        fn pin(&self) -> usize {
            (self.port_pin() & 0x0f) as usize
        }

        #[inline]
        fn block(&self) -> &'static pac::gpioa::RegisterBlock {
            match self.port() {
                GpioPort::GPIOA => unsafe { pac::GPIOA::PTR.as_ref().unwrap() },
                GpioPort::GPIOB => unsafe {
                    (pac::GPIOB::PTR as *const pac::gpioa::RegisterBlock)
                        .as_ref()
                        .unwrap()
                },
                GpioPort::GPIOF => unsafe {
                    (pac::GPIOF::PTR as *const pac::gpioa::RegisterBlock)
                        .as_ref()
                        .unwrap()
                },
            }
        }

        #[inline]
        fn set_mode(&self, mode: PinMode) {
            let block = self.block();

            block.moder.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<2>(
                    self.pin() * 2,
                    r.bits(),
                    mode as u32,
                ))
            })
        }

        #[inline]
        fn set_output_type(&self, output_type: PinOutputType) {
            self.block().otyper.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(
                    self.pin(),
                    r.bits(),
                    output_type as u32,
                ))
            })
        }

        #[inline]
        fn set_io_type(&self, io_type: PinIoType) {
            let (pushpull, output_type) = io_type.split();
            self.set_push_pull(pushpull);
            self.set_output_type(output_type)
        }

        #[inline]
        fn set_push_pull(&self, push_pull: PinPullUpDown) {
            self.block().pupdr.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<2>(
                    self.pin() * 2,
                    r.bits(),
                    push_pull as u32,
                ))
            })
        }

        #[inline]
        fn read(&self) -> PinLevel {
            let r = self.block().idr.read().bits();
            bit_mask_idx_get::<1>(self.pin(), r).into()
        }

        #[inline]
        fn write(&self, level: PinLevel) {
            self.block().odr.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(self.pin(), r.bits(), level as u32))
            })
        }

        #[inline]
        fn lock(&self, _lock: bool) {
            todo!()
        }

        #[inline]
        fn set_af(&self, af: PinAF) {
            let block = self.block();

            if self.pin() < 8 {
                block.afrl.modify(|r, w| unsafe {
                    w.bits(bit_mask_idx_modify::<4>(
                        self.pin() * 4,
                        r.bits(),
                        af as u32,
                    ))
                })
            } else {
                block.afrh.modify(|r, w| unsafe {
                    w.bits(bit_mask_idx_modify::<4>(
                        (self.pin() - 8) * 4,
                        r.bits(),
                        af as u32,
                    ))
                })
            }
        }

        #[inline]
        fn clear(&self) {
            self.block()
                .bsrr
                .write(|w| unsafe { w.bits(bit_mask_idx::<1>(self.pin() + 16)) })
        }

        #[inline]
        fn set(&self) {
            self.block()
                .bsrr
                .write(|w| unsafe { w.bits(bit_mask_idx::<1>(self.pin())) })
        }

        #[inline]
        fn reset(&self) {
            self.block()
                .brr
                .write(|w| unsafe { w.bits(1 << self.pin()) })
        }

        #[inline]
        fn set_speed(&self, speed: PinSpeed) {
            self.block().ospeedr.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<2>(
                    self.pin() * 2,
                    r.bits(),
                    speed as u32,
                ))
            })
        }
    }
}
