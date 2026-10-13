pub(super) mod sealed {
    use super::super::*;
    use crate::clock::sys_pclk;
    use crate::delay::wait_for_true_timeout_block;
    use crate::i2c::Id;
    use crate::pac;
    pub const WAIT_FLAG_TIMEOUT: usize = 100_000;

    // Bus flag wait timeout: 100 ms.

    pub trait Instance {
        // Other MCUs might have multiple I2C peripherals in the future.
        fn id() -> Id;

        #[inline]
        fn block() -> &'static pac::i2c::RegisterBlock {
            match Self::id() {
                Id::I2c1 => unsafe { pac::I2C::PTR.as_ref().unwrap() },
            }
        }

        /// Enable configuration.
        fn enable_config(en: bool) {
            Self::block().cr1.modify(|_, w| w.pe().bit(en))
        }

        /// Reset the I2C peripheral.
        #[inline]
        fn reset() {
            Self::id().clock().reset()
        }

        /// Check whether the bus is released (software reset bit).
        #[inline]
        fn is_bus_release() -> bool {
            Self::block().cr1.read().swrst().bit()
        }

        /// Generate a start signal.
        #[inline]
        fn start() {
            Self::block().cr1.modify(|_, w| w.start().set_bit())
        }

        /// Generate a stop signal.
        #[inline]
        fn stop() {
            Self::block().cr1.modify(|_, w| w.stop().set_bit())
        }

        /// Software reset.
        /// When set, I2C is in the reset state. Before releasing the reset, make sure
        /// the I2C pins are released and the bus is idle.
        /// - 0: I2C module is not in the reset state.
        /// - 1: I2C module is in the reset state.
        ///   Note: this bit can be used to re-initialize I2C after an error or lock-up,
        ///   e.g. when BUSY=1 but no stop condition is detected on the bus.
        #[inline]
        fn soft_reset() {
            Self::block().cr1.modify(|_, w| w.swrst().set_bit())
        }

        /// Write data into the data register.
        #[inline]
        fn transmit(data: u8) {
            Self::block().dr.modify(|_, w| unsafe { w.dr().bits(data) });
        }

        /// Set ACK or NACK.
        #[inline]
        fn ack(is_ack: bool) {
            Self::block().cr1.modify(|_, w| w.ack().bit(is_ack))
        }

        /// Write the slave address register.
        #[inline]
        fn address(address: u8) {
            Self::block()
                .oar1
                .modify(|_, w| unsafe { w.add().bits(address) });
        }

        /// Enable general call (CR1.ENGC).
        #[inline]
        fn set_general_call(en: bool) {
            Self::block().cr1.modify(|_, w| w.engc().bit(en));
        }

        /// Disable clock stretching (CR1.NOSTRETCH).
        #[inline]
        fn set_no_stretch(en: bool) {
            Self::block().cr1.modify(|_, w| w.nostretch().bit(en));
        }

        /// Enable PEC check (CR1.ENPEC).
        #[inline]
        fn set_pec(en: bool) {
            Self::block().cr1.modify(|_, w| w.enpec().bit(en));
        }

        /// Enable DMA request (CR2.DMAEN).
        #[inline]
        fn dma_enable(en: bool) {
            Self::block().cr2.modify(|_, w| w.dmaen().bit(en));
        }

        /// The next DMA transfer is the last one (CR2.LAST).
        #[inline]
        fn set_last_dma(en: bool) {
            Self::block().cr2.modify(|_, w| w.last().bit(en));
        }

        /// Clear the address-matched flag (ADDR): read SR1 then SR2.
        #[inline]
        fn clear_addr() {
            let _ = Self::block().sr1.read().bits();
            let _ = Self::block().sr2.read().bits();
        }

        /// Whether the address is a general-call address (SR2.GENCALL).
        #[inline]
        fn is_general_call() -> bool {
            Self::block().sr2.read().gencall().bit()
        }

        /// Whether currently transmitting or receiving (SR2.TRA): true means transmitting.
        #[inline]
        fn is_transmitter() -> bool {
            Self::block().sr2.read().tra().bit()
        }

        /// Bus busy.
        #[inline]
        fn busy() -> bool {
            Self::block().sr2.read().busy().bit()
        }

        /// Enable or disable the event interrupt.
        #[inline]
        fn event_config(event: Event, en: bool) {
            let cr2 = &Self::block().cr2;
            match event {
                Event::SB | Event::ADD | Event::STOPF | Event::BTF => {
                    cr2.modify(|_, w| w.itevten().bit(en))
                }
                Event::RXNE => cr2.modify(|_, w| w.itbufen().bit(en)),
                Event::TXE => cr2.modify(|_, w| w.itbufen().bit(en)),
                Event::BERR | Event::ARLO | Event::AF | Event::OVR | Event::PECERR => {
                    cr2.modify(|_, w| w.iterren().bit(en))
                }
            }
        }

        /// Return whether the interrupt event was matched,
        /// i.e. the event interrupt is enabled and the event flag is set.
        #[inline]
        fn is_event_match(event: Event) -> bool {
            let sr1 = Self::block().sr1.read();
            let cr2 = Self::block().cr2.read();
            match event {
                Event::SB => sr1.sb().bit() && cr2.itevten().bit(),
                Event::ADD => sr1.addr().bit() && cr2.itevten().bit(),
                Event::STOPF => sr1.stopf().bit() && cr2.itevten().bit(),
                Event::BTF => sr1.btf().bit() && cr2.itevten().bit(),
                Event::RXNE => sr1.rx_ne().bit() && cr2.itbufen().bit(),
                Event::TXE => sr1.tx_e().bit() && cr2.itbufen().bit(),
                Event::BERR => sr1.berr().bit() && cr2.iterren().bit(),
                Event::ARLO => sr1.arlo().bit() && cr2.iterren().bit(),
                Event::AF => sr1.af().bit() && cr2.iterren().bit(),
                Event::OVR => sr1.ovr().bit() && cr2.iterren().bit(),
                Event::PECERR => sr1.pecerr().bit() && cr2.iterren().bit(),
            }
        }

        #[inline]
        fn event_flag(event: Event) -> bool {
            let sr1 = Self::block().sr1.read();
            match event {
                Event::SB => sr1.sb().bit(),
                Event::ADD => sr1.addr().bit(),
                Event::STOPF => sr1.stopf().bit(),
                Event::BTF => sr1.btf().bit(),
                Event::RXNE => sr1.rx_ne().bit(),
                Event::TXE => sr1.tx_e().bit(),
                Event::BERR => sr1.berr().bit(),
                Event::ARLO => sr1.arlo().bit(),
                Event::AF => sr1.af().bit(),
                Event::OVR => sr1.ovr().bit(),
                Event::PECERR => sr1.pecerr().bit(),
            }
        }

        #[inline]
        fn event_clear(event: Event) {
            Self::block().sr1.modify(|r, w| match event {
                Event::SB => {
                    // Cleared by reading I2C_SR1 then writing the data register;
                    // or cleared by hardware when PE=0.
                    w
                }
                Event::ADD => {
                    // Cleared by reading I2C_SR1 then reading I2C_SR2; or cleared by
                    // hardware when PE=0.
                    let r = r.bits();
                    let _ = Self::block().sr2.read();
                    unsafe { w.bits(r) }
                }
                Event::STOPF => {
                    // Cleared by reading I2C_SR1 then writing I2C_CR1, or by hardware
                    // when PE=0.
                    w
                }
                Event::BTF => {
                    // Cleared by reading I2C_SR1 then reading or writing the data
                    // register, or after a start/stop condition, or by hardware when PE=0.
                    w
                }
                Event::RXNE => {
                    // Set when the data register is not empty during reception (not set
                    // during address phase). Cleared by reading/writing the data register,
                    // or by hardware when PE=0.
                    // Note: when BTF is set, reading data does not clear RxNE because the
                    // data register is still full.
                    w
                }
                Event::TXE => {
                    // Cleared by writing the data register, or after a start/stop
                    // condition, or automatically by hardware when PE=0.
                    w
                }
                Event::BERR => w.berr().clear_bit(),
                Event::ARLO => w.arlo().clear_bit(),
                Event::AF => w.af().clear_bit(),
                Event::OVR => w.ovr().clear_bit(),
                Event::PECERR => w.pecerr().clear_bit(),
            });
        }

        /// ACK/PEC position (for data reception). Software can set/clear this register,
        /// or it is cleared by hardware when PE=0.
        ///
        /// - 0: the ACK bit controls (N)ACK of the byte currently in the shift
        ///   register; the PEC bit indicates the current shift-register byte is PEC.
        /// - 1: the ACK bit controls (N)ACK of the next byte to be received in the
        ///   shift register; the PEC bit indicates the next received byte is PEC.
        ///
        /// Note: the POS bit is only used in 2-byte receive configuration and must be
        /// set before receiving data. To NACK the 2nd byte, the ACK bit must be
        /// cleared after clearing ADDR. To detect the 2nd byte's PEC, the PEC bit must
        /// be set at the ADDR-stretch event after POS is configured.
        #[inline]
        fn clear_pos() {
            Self::block().cr1.modify(|_, w| w.pos().clear_bit());
        }

        fn master_transmit_block(address: u8, buf: &[u8]) -> Result<usize, Error> {
            // If the bus is busy, exit.
            // wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::busy() == false)
            //     .map_err(|_| Error::Busy)?;

            Self::clear_pos();

            Self::start();
            // SB=1: cleared by reading SR1 then writing the data register.
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::SB))
                .map_err(|_| {
                    Self::event_clear(Event::ARLO);
                    Error::Start
                })?;

            Self::transmit(address << 1);

            // ADDR=1: cleared by reading SR1 then SR2.
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::ADD))
                .map_err(|_| {
                    // Self::debug();
                    // Clear the AF flag.
                    Self::event_clear(Event::AF);
                    Self::stop();
                    Error::Address
                })?;
            Self::event_clear(Event::ADD);

            // The TRA bit indicates whether the master is in receiver or transmitter mode.

            let mut iter = buf.iter();
            if let Some(d) = iter.next() {
                // EV8_1: TxE=1, shift register empty, data register empty, write Data1 to DR.
                wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::TXE))
                    .map_err(|_| Error::Tx)?;
                Self::transmit(*d);
            }

            // Then send the remaining data.
            for t in iter {
                // EV8: TxE=1, shift register not empty, data register empty, write Data2
                // to DR, this bit is cleared.
                wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::TXE))
                    .map_err(|_| {
                        Self::stop();
                        Error::Tx
                    })?;
                Self::transmit(*t);
            }

            // EV8_2: TxE=1, BTF=1, write the Stop bit; when hardware sends the Stop bit,
            // TxE and BTF are cleared.
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::TXE))
                .map_err(|_| {
                    Self::stop();
                    Error::Tx
                })?;

            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::TXE))
                .map_err(|_| {
                    Self::stop();
                    Error::Tx
                })?;

            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::BTF))
                .map_err(|_| {
                    Self::stop();
                    Error::Tx
                })?;

            Self::stop();

            Ok(buf.len())
        }

        fn master_receive_block(address: u8, buf: &mut [u8]) -> Result<usize, Error> {
            let block = Self::block();
            Self::clear_pos();

            Self::start();
            // EV5: SB=1, read SR1 then write DR to clear this bit.
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::SB))
                .map_err(|_| Error::Start)?;

            Self::transmit((address << 1) | 1);

            // EV6: ADDR, read SR1 then SR2 to clear this bit.
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::ADD))
                .map_err(|_| Error::Address)?;
            Self::event_clear(Event::ADD);

            let len = buf.len();

            let mut enumerate = buf.iter_mut().enumerate();
            while let Some((idx, p)) = enumerate.next() {
                let remain = len - idx;
                if remain > 2 {
                    Self::ack(true);
                    // EV7: RxNE=1, read DR to clear this bit.
                    wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || {
                        Self::event_flag(Event::RXNE)
                    })
                    .map_err(|_| Error::RX)?;

                    // Read the data.
                    *p = block.dr.read().dr().bits();

                    if Self::event_flag(Event::BTF) {
                        Self::ack(true);
                        let (_, p) = enumerate.next().unwrap();
                        *p = block.dr.read().dr().bits();
                    }
                } else if remain == 2 {
                    Self::ack(false);

                    // EV7: RxNE=1, read DR to clear this bit.
                    wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || {
                        Self::event_flag(Event::RXNE)
                    })
                    .map_err(|_| Error::RX)?;
                    // Read the data.
                    *p = block.dr.read().dr().bits();
                    if Self::event_flag(Event::BTF) {
                        Self::ack(false);
                        let (_, p) = enumerate.next().unwrap();
                        *p = block.dr.read().dr().bits();
                    }
                } else if remain == 1 {
                    Self::ack(false);
                    // EV7: RxNE=1, read DR to clear this bit.
                    wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || {
                        Self::event_flag(Event::RXNE)
                    })
                    .map_err(|_| Error::RX)?;
                    // Read the data.
                    *p = block.dr.read().dr().bits();
                }
            }

            Self::stop();

            Ok(buf.len())
        }

        fn config(config: Config) -> Result<(), Error> {
            let block = Self::block();

            Self::enable_config(false);

            let plk = sys_pclk();
            // In standard mode: 2 MHz; in fast mode: 4 MHz.
            // The I2C module clock must match the PCLK in Hz.
            // Standard mode: freq >= 4.
            // Fast mode: freq >= 12.
            let freq = plk / 1000 / 1000;

            // freq
            block
                .cr2
                .modify(|_, w| unsafe { w.freq().bits(freq as u8) });

            if config.speed > SPEED_HZ_FAST {
                return Err(Error::SpeedMode);
            }

            let ccr = if config.speed <= SPEED_HZ_STAND {
                let ccr = plk / 2 / config.speed as u32;
                if ccr <= 0x04 {
                    return Err(Error::PClock);
                }
                // F/S bit, false: standard mode, true: fast mode.
                block.ccr.modify(|_, w| w.f_s().bit(false));
                ccr
            } else {
                // config.speed <= SPEED_HZ_FAST
                let ccr = plk / 3 / config.speed as u32;
                let (ccr, duty) = if ccr >= 1 {
                    (ccr, false)
                } else {
                    let ccr = plk / 15 / config.speed as u32;
                    if ccr > 1 {
                        (ccr, true)
                    } else {
                        return Err(Error::PClock);
                    }
                };

                // F/S bit, false: standard mode, true: fast mode.
                block.ccr.modify(|_, w| w.f_s().bit(true));
                // The duty bit is only valid in fast mode.
                block.ccr.modify(|_, w| w.duty().bit(duty));
                ccr
            };

            // Set the clock divider parameter.
            block.ccr.modify(|_, w| unsafe { w.ccr().bits(ccr as u16) });

            let rise = if config.speed <= SPEED_HZ_STAND {
                freq + 1
            } else {
                freq * 300 / 1000 + 1
            } as u8;
            block.trise.modify(|_, w| unsafe { w.trise().bits(rise) });

            // Own slave address, general call, clock stretching and PEC.
            Self::address(config.own_address);
            Self::set_general_call(config.general_call);
            Self::set_no_stretch(config.no_stretch);
            Self::set_pec(config.pec);

            Self::enable_config(true);

            Ok(())
        }
    }
}
