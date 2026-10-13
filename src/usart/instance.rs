pub mod sealed {
    use super::super::*;
    use crate::clock;
    use crate::pac;

    pub trait Instance {
        fn id() -> Id;

        /// USART1 / USART2 share the `usart1` register-block layout; only the base address differs.
        #[inline]
        fn block() -> &'static pac::usart1::RegisterBlock {
            match Self::id() {
                Id::USART1 => unsafe { pac::USART1::PTR.as_ref().unwrap() },
                Id::USART2 => unsafe { pac::USART2::PTR.as_ref().unwrap() },
            }
        }


        #[inline]
        fn enable() {
            Self::id().clock().open()
        }

        #[inline]
        fn disable() {
            Self::id().clock().close()
        }

        #[inline]
        fn reset() {
            Self::id().clock().reset()
        }

        #[inline]
        fn start() {
            Self::block().cr1.modify(|_, w| w.ue().bit(true))
        }

        #[inline]
        fn stop() {
            Self::block().cr1.modify(|_, w| w.ue().bit(false))
        }


        #[inline]
        fn read() -> u8 {
            Self::block().dr.read().dr().bits() as u8
        }

        /// Read 9-bit data (DR is 9 bits wide, used in 9-bit data mode).
        #[inline]
        fn read_u16() -> u16 {
            Self::block().dr.read().dr().bits()
        }

        #[inline]
        fn write(data: u8) {
            Self::block()
                .dr
                .write(|w| unsafe { w.dr().bits(data as u16) });
        }

        /// Write 9-bit data (low 9 bits valid).
        #[inline]
        fn write_u16(data: u16) {
            Self::block()
                .dr
                .write(|w| unsafe { w.dr().bits(data & 0x01ff) });
        }

        #[inline]
        fn rx_ready() -> bool {
            Self::event_flag(Event::RXNE)
        }

        #[inline]
        fn tx_ready() -> bool {
            Self::event_flag(Event::TXE)
        }

        #[inline]
        fn read_byte_blocking() -> u8 {
            while !Self::event_flag(Event::RXNE) {}
            Self::read()
        }

        #[inline]
        fn read_bytes_blocking(buf: &mut [u8]) -> usize {
            let len = buf.len();
            for item in buf {
                *item = Self::read_byte_blocking()
            }
            len
        }

        fn read_bytes_idle_blocking(buf: &mut [u8]) -> usize {
            let mut cnt = 0;
            for item in buf {
                loop {
                    if Self::event_flag(Event::RXNE) {
                        break;
                    }
                    // Line idle: this transfer has ended.
                    if Self::event_flag(Event::IDLE) {
                        // Read SR then DR to clear IDLE; the byte read is invalid.
                        Self::event_clear(Event::IDLE);
                        return cnt;
                    }
                }
                cnt += 1;
                *item = Self::read();
            }
            cnt
        }

        #[inline]
        fn write_byte_blocking(data: u8) -> Result<(), Error> {
            while !Self::event_flag(Event::TXE) {}
            Self::write(data);
            Ok(())
        }

        #[inline]
        fn write_bytes_blocking(buf: &[u8]) -> Result<(), Error> {
            for item in buf {
                Self::write_byte_blocking(*item)?;
            }
            Ok(())
        }

        #[inline]
        fn tx_flush() {
            while !Self::event_flag(Event::TC) {}
        }

        /// Send a break character (set CR1.SBK).
        #[inline]
        fn send_break() {
            Self::block().cr1.modify(|_, w| w.sbk().set_bit());
        }

        /// Stop sending the break character.
        #[inline]
        fn clear_break() {
            Self::block().cr1.modify(|_, w| w.sbk().clear_bit());
        }


        #[inline]
        fn rx_enable(en: bool) {
            Self::block().cr1.modify(|_, w| w.re().bit(en))
        }

        #[inline]
        fn tx_enable(en: bool) {
            Self::block().cr1.modify(|_, w| w.te().bit(en))
        }

        #[inline]
        fn rts_enable(en: bool) {
            Self::block().cr3.modify(|_, w| w.rtse().bit(en))
        }

        #[inline]
        fn cts_enable(en: bool) {
            Self::block().cr3.modify(|_, w| w.ctse().bit(en))
        }

        /// Set RTS / CTS together (CR3.RTSE / CR3.CTSE).
        #[inline]
        fn set_flow_control(rts: bool, cts: bool) {
            Self::block()
                .cr3
                .modify(|_, w| w.ctse().bit(cts).rtse().bit(rts))
        }


        #[inline]
        fn rx_dma_enable(en: bool) {
            Self::block().cr3.modify(|_, w| w.dmar().bit(en));
        }

        #[inline]
        fn tx_dma_enable(en: bool) {
            Self::block().cr3.modify(|_, w| w.dmat().bit(en));
        }


        /// Half-duplex (single-wire) mode: CR3.HDSEL.
        #[inline]
        fn set_half_duplex(en: bool) {
            Self::block().cr3.modify(|_, w| w.hdsel().bit(en));
        }

        /// IrDA mode: `Some(mode)` enables CR3.IREN and sets the low-power bit IRLP.
        #[inline]
        fn set_irda(mode: Option<IrdaMode>) {
            let en = mode.is_some();
            let low_power = matches!(mode, Some(IrdaMode::LowPower));
            Self::block()
                .cr3
                .modify(|_, w| w.iren().bit(en).irlp().bit(low_power));
        }

        /// Synchronous-mode clock output (CR2.CLKEN / CPOL / CPHA / LBCL).
        #[inline]
        fn set_sync_clock(cfg: Option<SyncClockConfig>) {
            Self::block().cr2.modify(|_, w| {
                let w = w.clken().bit(cfg.is_some());
                match cfg {
                    Some(c) => w
                        .cpol()
                        .bit(c.polarity == ClockPolarity::IdleHigh)
                        .cpha()
                        .bit(c.phase == ClockPhase::SecondEdge)
                        .lbcl()
                        .bit(c.last_bit_clock),
                    None => w
                        .cpol()
                        .bit(false)
                        .cpha()
                        .bit(false)
                        .lbcl()
                        .bit(false),
                }
            });
        }

        /// Mute mode (CR1.RWU) and wake-up method (CR1.WAKE).
        #[inline]
        fn set_mute(mode: Option<WakeMethod>) {
            Self::block().cr1.modify(|_, w| {
                w.rwu()
                    .bit(mode.is_some())
                    .wake()
                    .bit(matches!(mode, Some(WakeMethod::AddressMark)))
            });
        }

        /// Multiprocessor communication node address (CR2.ADD).
        #[inline]
        fn set_address(address: u8) {
            Self::block()
                .cr2
                .modify(|_, w| unsafe { w.add().bits(address) });
        }


        /// Enable auto-baud-rate detection and set the measurement mode (CR3.ABREN / ABRMOD).
        #[inline]
        fn set_auto_baud(mode: Option<AutoBaudMode>) {
            Self::block().cr3.modify(|_, w| {
                w.abren().bit(mode.is_some());
                unsafe { w.abrmod().bits(mode.unwrap_or_default() as u8) }
            });
        }

        /// Request a new auto-baud-rate measurement (write 1 to SR.ABRRQ).
        #[inline]
        fn request_auto_baud() {
            Self::block().sr.modify(|_, w| w.abrrq().set_bit());
        }

        /// Read the BRR value corresponding to the auto-baud-rate measurement result.
        #[inline]
        fn auto_baud_result() -> u16 {
            Self::block().brr.read().bits() as u16
        }


        /// Clear the event flag.
        fn event_clear(event: Event) {
            Self::block().sr.modify(|r, w| match event {
                // PE / FE / NE / ORE / IDLE: must read SR then DR to clear.
                Event::PE | Event::FE | Event::NE | Event::ORE | Event::IDLE => {
                    let _ = r.bits();
                    let _ = Self::read();
                    w
                }
                Event::RXNE => w.rxne().clear_bit(),
                Event::TC => w.tc().clear_bit(),
                // TXE is cleared by writing DR.
                Event::TXE => w,
                Event::CTS => w.cts().clear_bit(),
                // ABRE: write 1 to ABRRQ to request a re-measurement.
                Event::ABRE => w.abrrq().set_bit(),
                // ABRF is cleared by hardware on the next measurement.
                Event::ABRF => w,
            });
        }

        /// Return the event flag.
        fn event_flag(event: Event) -> bool {
            let sr = Self::block().sr.read();
            match event {
                Event::PE => sr.pe(),
                Event::FE => sr.fe(),
                Event::NE => sr.ne(),
                Event::ORE => sr.ore(),
                Event::IDLE => sr.idle(),
                Event::RXNE => sr.rxne(),
                Event::TC => sr.tc(),
                Event::TXE => sr.txe(),
                Event::CTS => sr.cts(),
                Event::ABRE => sr.abre(),
                Event::ABRF => sr.abrf(),
            }
            .bit()
        }

        /// Enable or disable the event interrupt.
        fn event_config(event: Event, en: bool) {
            let block = Self::block();
            match event {
                Event::PE => block.cr1.modify(|_, w| w.peie().bit(en)),
                // FE / NE / ORE share CR3.EIE.
                Event::FE | Event::NE | Event::ORE => {
                    block.cr3.modify(|_, w| w.eie().bit(en));
                }
                Event::IDLE => block.cr1.modify(|_, w| w.idleie().bit(en)),
                Event::RXNE => block.cr1.modify(|_, w| w.rxneie().bit(en)),
                Event::TC => block.cr1.modify(|_, w| w.tcie().bit(en)),
                Event::TXE => block.cr1.modify(|_, w| w.txeie().bit(en)),
                // CTS interrupt: CR3.CTSIE.
                Event::CTS => block.cr3.modify(|_, w| w.ctsie().bit(en)),
                // ABRE / ABRF have no dedicated interrupt-enable bit; they are set by
                // hardware along with the ABR process.
                Event::ABRE => {}
                Event::ABRF => {}
            }
        }

        /// Return whether the event interrupt is enabled.
        fn is_event_enable(event: Event) -> bool {
            let block = Self::block();
            match event {
                Event::PE => block.cr1.read().peie().bit(),
                // These three share CR3.EIE.
                Event::FE | Event::NE | Event::ORE => block.cr3.read().eie().bit(),
                Event::IDLE => block.cr1.read().idleie().bit(),
                Event::RXNE => block.cr1.read().rxneie().bit(),
                Event::TC => block.cr1.read().tcie().bit(),
                Event::TXE => block.cr1.read().txeie().bit(),
                Event::CTS => block.cr3.read().ctsie().bit(),
                Event::ABRE => false,
                Event::ABRF => false,
            }
        }


        /// Compute BRR using integer arithmetic to avoid floating-point.
        ///
        /// With 16x oversampling: `USARTDIV = f_PCLK / (16 * baud)`.
        /// With 8x oversampling:  `USARTDIV = f_PCLK / (8 * baud)` (only the low 3
        /// fraction bits are valid, bit3 stays 0).
        fn set_baud_rate(baud_rate: u32, over_sampling: OverSampling) {
            let pclk = clock::sys_pclk() as u64;
            let mul: u64 = if over_sampling == OverSampling::Eight {
                2
            } else {
                1
            };

            // USARTDIV is measured in 1/16 units, with rounding.
            let div16 = (pclk * mul + baud_rate as u64 / 2) / baud_rate as u64;
            let mantissa = (div16 / 16) as u16;
            let mut fraction = (div16 % 16) as u8;
            if over_sampling == OverSampling::Eight {
                fraction &= 0x7;
            }

            Self::block().brr.modify(|_, w| unsafe {
                w.div_mantissa()
                    .bits(mantissa)
                    .div_fraction()
                    .bits(fraction)
            });
        }


        /// Configure the USART (must be written while UE=0, so stop first then start).
        fn config(config: Config) {
            let block = Self::block();

            Self::stop();

            // Stop bits.
            block
                .cr2
                .modify(|_, w| unsafe { w.stop().bits(config.stop_bit as u8) });

            // Data bits + parity enable + parity polarity (PS: 0 even, 1 odd).
            block.cr1.modify(|_, w| {
                w.m()
                    .bit(config.data_bits.into())
                    .pce()
                    .bit(config.parity.is_enable())
                    .ps()
                    .bit(config.parity == Parity::Odd)
            });

            // Oversampling.
            block
                .cr3
                .modify(|_, w| w.over8().bit(config.over_sampling.into()));

            // Hardware flow control.
            Self::set_flow_control(config.hw_flow_ctrl.rts(), config.hw_flow_ctrl.cts());

            // Auto baud rate.
            Self::set_auto_baud(config.auto_baud);

            // Half-duplex.
            Self::set_half_duplex(config.half_duplex);

            // IrDA.
            Self::set_irda(config.irda);

            // Synchronous clock output.
            Self::set_sync_clock(config.sync_clock);

            // Mute mode & wake-up method + node address.
            Self::set_mute(config.mute);
            Self::set_address(config.address);

            // Baud rate.
            Self::set_baud_rate(config.baud_rate.0, config.over_sampling);

            Self::start();
        }
    }
}
