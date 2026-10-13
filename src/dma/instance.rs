pub(crate) mod sealed {
    use enumset::__internal::EnumSetTypeRepr;

    use super::super::*;
    use crate::pac;

    pub(crate) trait Instance {
        fn id() -> Id;

        #[inline]
        fn block() -> &'static pac::dma::RegisterBlock {
            match Self::id() {
                Id::DMA => unsafe { pac::DMA::PTR.as_ref().unwrap() },
            }
        }

        #[inline]
        fn enable(channel: Channel, en: bool) {
            match channel {
                Channel::Channel1 => Self::block().ccr1.modify(|_, w| w.en().bit(en)),
                Channel::Channel2 => Self::block().ccr2.modify(|_, w| w.en().bit(en)),
                Channel::Channel3 => Self::block().ccr3.modify(|_, w| w.en().bit(en)),
            }
        }

        // #[inline]
        // fn is_cycle_mode(channel: Channel) -> bool {
        //     let block = Self::block();
        //     match channel {
        //         Channel::Channel1 => block.ccr1.read().circ().bit(),
        //         Channel::Channel2 => block.ccr2.read().circ().bit(),
        //         Channel::Channel3 => block.ccr2.read().circ().bit(),
        //     }
        // }

        // Read the remaining transfer count.
        fn remain_count(channel: Channel) -> u16 {
            let block = Self::block();
            let cnt = match channel {
                Channel::Channel1 => block.cndtr1.read().bits(),
                Channel::Channel2 => block.cndtr2.read().bits(),
                Channel::Channel3 => block.cndtr3.read().bits(),
            };
            cnt as u16
        }

        fn config(channel: Channel, config: Config) {
            let block = Self::block();

            Self::enable(channel, false);

            match channel {
                Channel::Channel1 => {
                    block.ccr1.modify(|_, w| unsafe {
                        w.mem2mem()
                            .bit(config.diretion == Direction::MemoryToMemory)
                            .pl() // priority
                            .bits(config.prioritie as u8)
                            .msize() // memory data width
                            .bits(config.memDataSize as u8)
                            .psize() // peripheral data width
                            .bits(config.periphDataSize as u8)
                            .minc() // memory address increment enable
                            .bit(config.memInc)
                            .pinc() // peripheral address increment enable
                            .bit(config.periphInc)
                            .dir() // data transfer direction, 0: read from peripheral, 1: read from memory
                            .bit(config.diretion != Direction::PeriphToMemory)
                    });

                    block.cpar1.write(|w| unsafe { w.bits(config.periphAddr) });
                    block.cmar1.write(|w| unsafe { w.bits(config.memAddr) });

                    // The transfer count is 0~65535. This register is only written while the
                    // channel is disabled (DMA_CCRx.EN=0). Once the channel is enabled it
                    // becomes read-only and indicates the remaining byte count, which
                    // decrements after each transfer. At the end of the transfer the value
                    // becomes 0, or, when the channel is configured in circular mode, is
                    // automatically reloaded with its original value. When it is 0, no data
                    // is transferred even if the channel starts.
                    match config.mode {
                        RepeatMode::OneTime(cnt) => {
                            block.ccr1.modify(|_, w| w.circ().bit(false));
                            block.cndtr1.modify(|_, w| unsafe { w.ndt().bits(cnt) });
                        }
                        RepeatMode::Repeat(cnt) => {
                            block.ccr1.modify(|_, w| w.circ().bit(true));
                            block.cndtr1.modify(|_, w| unsafe { w.ndt().bits(cnt) });
                        }
                    };
                }
                Channel::Channel2 => {
                    block.ccr2.modify(|_, w| unsafe {
                        w.mem2mem()
                            .bit(config.diretion == Direction::MemoryToMemory)
                            .pl() // priority
                            .bits(config.prioritie as u8)
                            .msize() // memory data width
                            .bits(config.memDataSize as u8)
                            .psize() // peripheral data width
                            .bits(config.periphDataSize as u8)
                            .minc() // memory address increment enable
                            .bit(config.memInc)
                            .pinc() // peripheral address increment enable
                            .bit(config.periphInc)
                            .dir() // data transfer direction, 0: read from peripheral, 1: read from memory
                            .bit(config.diretion != Direction::PeriphToMemory)
                    });

                    block.cpar2.write(|w| unsafe { w.bits(config.periphAddr) });
                    block.cmar2.write(|w| unsafe { w.bits(config.memAddr) });

                    // The transfer count is 0~65535. This register is only written while the
                    // channel is disabled (DMA_CCRx.EN=0). Once the channel is enabled it
                    // becomes read-only and indicates the remaining byte count, which
                    // decrements after each transfer. At the end of the transfer the value
                    // becomes 0, or, when the channel is configured in circular mode, is
                    // automatically reloaded with its original value. When it is 0, no data
                    // is transferred even if the channel starts.
                    match config.mode {
                        RepeatMode::OneTime(cnt) => {
                            block.ccr2.modify(|_, w| w.circ().bit(false));
                            block.cndtr2.modify(|_, w| unsafe { w.ndt().bits(cnt) });
                        }
                        RepeatMode::Repeat(cnt) => {
                            block.ccr2.modify(|_, w| w.circ().bit(true));
                            block.cndtr2.modify(|_, w| unsafe { w.ndt().bits(cnt) });
                        }
                    };
                }
                Channel::Channel3 => {
                    block.ccr3.modify(|_, w| unsafe {
                        w.mem2mem()
                            .bit(config.diretion == Direction::MemoryToMemory)
                            .pl() // priority
                            .bits(config.prioritie as u8)
                            .msize() // memory data width
                            .bits(config.memDataSize as u8)
                            .psize() // peripheral data width
                            .bits(config.periphDataSize as u8)
                            .minc() // memory address increment enable
                            .bit(config.memInc)
                            .pinc() // peripheral address increment enable
                            .bit(config.periphInc)
                            .dir() // data transfer direction, 0: read from peripheral, 1: read from memory
                            .bit(config.diretion != Direction::PeriphToMemory)
                    });

                    block.cpar3.write(|w| unsafe { w.bits(config.periphAddr) });
                    block.cmar3.write(|w| unsafe { w.bits(config.memAddr) });

                    // The transfer count is 0~65535. This register is only written while the
                    // channel is disabled (DMA_CCRx.EN=0). Once the channel is enabled it
                    // becomes read-only and indicates the remaining byte count, which
                    // decrements after each transfer. At the end of the transfer the value
                    // becomes 0, or, when the channel is configured in circular mode, is
                    // automatically reloaded with its original value. When it is 0, no data
                    // is transferred even if the channel starts.
                    match config.mode {
                        RepeatMode::OneTime(cnt) => {
                            block.ccr3.modify(|_, w| w.circ().bit(false));
                            block.cndtr3.modify(|_, w| unsafe { w.ndt().bits(cnt) });
                        }
                        RepeatMode::Repeat(cnt) => {
                            block.ccr3.modify(|_, w| w.circ().bit(true));
                            block.cndtr3.modify(|_, w| unsafe { w.ndt().bits(cnt) });
                        }
                    };
                }
            }

            // Self::enable(false);
        }

        /// Clear the event flag.
        fn event_clear(channel: Channel, event: Event) {
            Self::block()
                .ifcr
                .write(|w| unsafe { w.bits(1 << (event as usize + (channel as usize * 4))) });
        }

        /// Return the event flag.
        fn event_flag(channel: Channel, event: Event) -> bool {
            Self::block()
                .isr
                .read()
                .bits()
                .has_bit((event as u32) + (channel as u32 * 4))
        }

        /// Enable or disable the event interrupt.
        #[allow(dead_code)]
        fn event_config(channel: Channel, event: Event, en: bool) {
            let block = Self::block();

            match channel {
                Channel::Channel1 => match event {
                    Event::GIF => {}
                    Event::HTIF => {
                        block.ccr1.modify(|_, w| w.htie().bit(en));
                    }
                    Event::TCIF => {
                        block.ccr1.modify(|_, w| w.tcie().bit(en));
                    }
                    Event::TEIF => {
                        block.ccr1.modify(|_, w| w.teie().bit(en));
                    }
                },
                Channel::Channel2 => match event {
                    Event::GIF => {}
                    Event::HTIF => {
                        block.ccr2.modify(|_, w| w.htie().bit(en));
                    }
                    Event::TCIF => {
                        block.ccr2.modify(|_, w| w.tcie().bit(en));
                    }
                    Event::TEIF => {
                        block.ccr2.modify(|_, w| w.teie().bit(en));
                    }
                },
                Channel::Channel3 => match event {
                    Event::GIF => {}
                    Event::HTIF => {
                        block.ccr3.modify(|_, w| w.htie().bit(en));
                    }
                    Event::TCIF => {
                        block.ccr3.modify(|_, w| w.tcie().bit(en));
                    }
                    Event::TEIF => {
                        block.ccr3.modify(|_, w| w.teie().bit(en));
                    }
                },
            }
        }

        /// Return whether the event interrupt is enabled.
        #[allow(dead_code)]
        fn is_event_enable(channel: Channel, event: Event) -> bool {
            let block = Self::block();

            match channel {
                Channel::Channel1 => match event {
                    Event::GIF => false,
                    Event::HTIF => block.ccr1.read().htie().bit(),
                    Event::TCIF => block.ccr1.read().tcie().bit(),
                    Event::TEIF => block.ccr1.read().teie().bit(),
                },
                Channel::Channel2 => match event {
                    Event::GIF => false,
                    Event::HTIF => block.ccr2.read().htie().bit(),
                    Event::TCIF => block.ccr2.read().tcie().bit(),
                    Event::TEIF => block.ccr2.read().teie().bit(),
                },
                Channel::Channel3 => match event {
                    Event::GIF => false,
                    Event::HTIF => block.ccr3.read().htie().bit(),
                    Event::TCIF => block.ccr3.read().tcie().bit(),
                    Event::TEIF => block.ccr3.read().teie().bit(),
                },
            }
        }
    }
}
