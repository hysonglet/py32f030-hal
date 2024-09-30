pub(super) mod sealed {
    use crate::pac;
    use crate::spi::*;
    pub trait Instance {
        fn id() -> Id;

        #[inline]
        fn block() -> &'static pac::spi1::RegisterBlock {
            match Self::id() {
                Id::SPI1 => unsafe { pac::SPI1::PTR.as_ref().unwrap() },
                Id::SPI2 => unsafe { pac::SPI2::PTR.as_ref().unwrap() },
            }
        }

        /// Bidirectional data mode enable
        #[inline]
        fn set_bidirectional_mode(mode: BidirectionalMode) {
            // 0: 2-line unidirectional data mode
            // 1: 1-line bidirectional data mode
            Self::block().cr1.modify(|_, w| {
                w.bidimode()
                    .bit(mode == BidirectionalMode::Line1Bidirectional)
            });
        }

        /// Output enable in bidirectional mode
        #[inline]
        fn enable_output_bidirectional_mode(en: bool) {
            // 0: Output disabled (receive-only mode)
            // 1: Output enabled (transmit-only mode)
            Self::block().cr1.modify(|_, w| w.bidioe().bit(en))
        }

        /// Receive control only.
        #[inline]
        fn enable_rx_readonly(en: bool) {
            // 0: Full-duplex (Transmit and receive)
            // 1: Output disabled (Receive-only mode)
            Self::block().cr1.modify(|_, w| w.rxonly().bit(en))
        }

        /// Software slave management
        ///
        /// When the SSM bit is set, the NSS pin input is replaced with the value from the SSI bit.
        #[inline]
        fn enable_soft_slave_management(en: bool) {
            // 0: Software slave management disabled
            // 1: Software slave management enabled
            Self::block().cr1.modify(|_, w| w.ssm().bit(en))
        }

        /// Internal slave select
        ///
        /// This bit has an effect only when the SSM bit is set. The value of this bit is forced onto the NSS pin and the I/O value of the NSS pin is ignored.
        #[inline]
        fn slave_enable(en: bool) {
            // nss pin level
            Self::block().cr1.modify(|_, w| w.ssi().bit(en));
        }

        /// Set Frame format
        #[inline]
        fn set_frame_format(format: BitOrder) {
            Self::block()
                .cr1
                .modify(|_, w| w.lsbfirst().bit(format == BitOrder::LSB));
        }

        /// SPI enable
        #[inline]
        fn spi_enable(en: bool) {
            // 0: SPI disabled
            // 1: SPI enable
            Self::block().cr1.modify(|_, w| w.spe().bit(en))
        }

        /// Master selection
        #[inline]
        fn set_rule(rule: Rule) {
            // 0: Slave configuration
            // 1: Master configuration
            // Note: This bit should *not* be changed when communication is ongoing.
            Self::block()
                .cr1
                .modify(|_, w| w.mstr().bit(rule == Rule::Master))
        }

        /// Clock polarity
        #[inline]
        fn set_clock_polarity(polarity: ClockPolarity) {
            // This bit should not be changed when communication is ongoing
            Self::block()
                .cr1
                .modify(|_, w| w.cpol().bit(polarity == ClockPolarity::Hight))
        }

        /// Clock phase
        #[inline]
        fn set_clock_phase(phase: ClockPhase) {
            Self::block()
                .cr1
                .modify(|_, w| w.cpha().bit(phase == ClockPhase::Hight))
        }

        /// Slave fast mode enable
        #[inline]
        fn set_slave_mode(mode: SlaveSpeedMode) {
            // Note: When the speed of SPI clock is less than pclk/4, this register bit must not be set.
            Self::block()
                .cr2
                .modify(|_, w| w.slvfm().bit(mode == SlaveSpeedMode::Fast))
        }

        /// SPI transmission data length
        #[inline]
        fn set_data_length(data_length: DataLength) {
            Self::block()
                .cr2
                .modify(|_, w| w.ds().bit(data_length == DataLength::Length16))
        }

        /// Tx buffer empty interrupt enable
        #[inline]
        fn enable_tx_empty_interrupt(en: bool) {
            // 0: TXE interrupt masked
            // 1: TXE interrupt not masked. Used to generate an interrupt request when the TXE flag is set
            Self::block().cr2.modify(|_, w| w.txeie().bit(en))
        }

        /// RX buffer not empty interrupt enable
        fn enable_rx_no_empty_interrupt(en: bool) {
            // 0: RXNE interrupt masked
            // 1: RXNE interrupt not masked. Used to generate an interrupt request when the RXNE flag is set
            Self::block().cr2.modify(|_, w| w.rxneie().bit(en))
        }

        /// Error interrupt enable
        #[inline]
        fn enable_error_interrupt(en: bool) {
            // 0: Error interrupt is masked
            // 1: Error interrupt is enabled
            // This bit controls the generation of an interrupt when an error condition occurs (CRCERR, VR, MODF in SPI mode).
            Self::block().cr2.modify(|_, w| w.errie().bit(en))
        }

        #[inline]
        fn set_baud_rate_div(div: BaudRateDiv) {
            Self::block()
                .cr1
                .modify(|_, w| unsafe { w.br().bits(div as u8) });
        }

        /// SS output enable
        #[inline]
        fn enable_ss_output(en: bool) {
            // 0: SS output is disabled in master mode and the SPI interface can work in multimaster configurtion
            // 1: SS output is enabled in master mode and when the SPI interface is enabled. The SPI interface cannot work in a multimaster environment.
            Self::block().cr2.modify(|_, w| w.ssoe().bit(en))
        }

        // dma option

        /// busy?
        #[inline]
        fn is_busy() -> bool {
            // Busy flag
            // 0: SPI (or I2S) not busy
            // 1: SPI (or I2S) is busy in communication or Tx buffer is not empty
            Self::block().sr.read().bsy().bit()
        }

        /// Overrun flag
        #[inline]
        fn is_overflow() -> bool {
            // 0: No overrun occurred
            // 1: Overrun occurred
            // This flag is set by hardware and reset by a software sequence.
            Self::block().sr.read().ovr().bit()
        }

        /// Transmit buffer empty
        #[inline]
        fn tx_empty() -> bool {
            // 0: Tx buffer not empty
            // 1: Tx buffer empty
            Self::block().sr.read().txe().bit()
        }

        /// Receive buffer not empty
        #[inline]
        fn rx_not_empty() -> bool {
            // 0: Rx buffer empty
            // 1: Rx buffer not empty
            Self::block().sr.read().rxne().bit()
        }

        #[inline]
        fn data_write(data: u16) {
            // The data register serves as an interface between the Rx and Tx FIFOs. When the data register is
            // read, RxFIFO is accessed while the write to data register accesses TxFIFO.
            // Note:Depending on the DS bit (data frame width selection), data transmission or reception is 8-bit or 16-bit.
            // For 8-bit data frames, the data registers are sent and received based on right -aligned 8-bit data.
            // When in receive mode, DR [15:8] is set to 0 by hardware.
            // For 16-bit data frame, the data register is 16-bit, and the entire DR [15:0] is used for transmit and receive.
            Self::block().dr.write(|w| unsafe { w.dr().bits(data) });
        }

        #[inline]
        fn data_read() -> u16 {
            // The data register serves as an interface between the Rx and Tx FIFOs. When the data register is
            // read, RxFIFO is accessed while the write to data register accesses TxFIFO.
            // Note:Depending on the DS bit (data frame width selection), data transmission or reception is 8-bit or 16-bit.
            // For 8-bit data frames, the data registers are sent and received based on right -aligned 8-bit data.
            // When in receive mode, DR [15:8] is set to 0 by hardware.
            // For 16-bit data frame, the data register is 16-bit, and the entire DR [15:0] is used for transmit and receive.
            Self::block().dr.read().dr().bits()
        }

        /// 返回 master 模式下spi的总线频率
        #[inline]
        fn get_baud_rate() -> u32 {
            let div: BaudRateDiv = Self::block().cr1.read().br().bits().into();

            div.baud_rate()
        }
    }
}
