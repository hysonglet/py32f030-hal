pub(super) mod sealed {
    use crate::clock::sys_pclk;
    use crate::delay::wait_for_flag_timeout;
    use crate::i2c::I2c;
    use crate::i2c::*;
    use crate::pac;

    // 总线标志等待超时， 100 ms
    const WAIT_FLAG_TIMEOUT_US: usize = 100_000;
    pub trait Instance {
        // 考虑以后其他单片机可能有多个IIC
        fn i2c() -> I2c;

        #[inline]
        fn block() -> &'static pac::i2c::RegisterBlock {
            match Self::i2c() {
                I2c::I2c1 => unsafe { pac::I2C::PTR.as_ref().unwrap() },
            }
        }

        // 使能配置
        fn enable_config(en: bool) {
            Self::block().cr1.modify(|_, w| w.pe().bit(en))
        }

        /// 外设时钟使能
        #[inline]
        fn enable(en: bool) {
            Self::i2c().enable(en)
        }

        #[inline]
        fn reset() {
            Self::i2c().reset()
        }

        fn bus_release() {
            Self::block().cr1.modify(|_, w| w.swrst().set_bit())
        }

        fn is_bus_release() -> bool {
            Self::block().cr1.read().swrst().bit()
        }

        #[inline]
        fn start() {
            Self::block().cr1.modify(|_, w| w.start().set_bit())
        }

        fn start_flag() -> bool {
            Self::block().sr1.read().sb().bit()
        }

        fn address_flag() -> bool {
            Self::block().sr1.read().addr().bit()
        }

        #[inline]
        fn stop() {
            Self::block().cr1.modify(|_, w| w.stop().set_bit())
        }

        #[inline]
        fn transmit(data: u8) {
            Self::block().dr.modify(|_, w| unsafe { w.dr().bits(data) });
        }

        /// 设置回复ack或nack
        #[inline]
        fn ack(is_ack: bool) {
            Self::block().cr1.modify(|_, w| w.ack().bit(is_ack))
        }

        #[inline]
        fn address(address: u8) {
            Self::block()
                .oar1
                .modify(|_, w| unsafe { w.add().bits(address) });
        }

        fn clear_address() {
            let block = Self::block();
            let _ = block.sr1.read();
            let _ = block.sr2.read();
        }

        /// 发送寄存器内容为空
        /// 在发送数据时，数据寄存器为空时该位被置 1，
        /// 在发送地址阶段不设置该位。
        /// 软件写数据到 DR 寄存器可清除该位，或在发生一个起始或停止条件后，或当 PE=0 时由硬件自动清除。如果收到一个 NACK，或下一个要发送的字节时PEC（PEC=1），该位不被置位。
        /// 注：在写入第 1 个要发送的数据后，或设置了BTF 时写入数据，都不能清除 TxE 位，因为此时数据寄存器为空。
        #[inline]
        fn tx_empty() -> bool {
            Self::block().sr1.read().tx_e().bit()
        }

        /// 数据寄存器非空（接收时）标志。
        /// 0：数据寄存器为空；
        /// 1：数据寄存器非空。
        /// 在接收时，当数据寄存器不为空，置位该寄存
        /// 器。在接收地址阶段，该寄存器不置位。
        /// 软件对数据寄存器的读写操作会清除该寄存器，
        /// 或当 PE=0 时由硬件清除。
        /// 注：当设置了 BTF 时，读取数据不能清除 RxNE
        /// 位，因为此时数据寄存器仍为满。
        #[inline]
        fn rx_not_empty() -> bool {
            Self::block().sr1.read().rx_ne().bit()
        }

        /// 字节传输结束标志位。
        /// 0：字节传输未完成   1：字节传输成功结束
        /// 在下列情况下硬件将置位该寄存器（当 slave 模式，NOSTRETCH=0 时；master 模式，与NOSTRETCH 无关）：— 接收时，当收到一个新字节（包括 ACK 脉冲 ） 且 数 据 寄 存 器 还 未 被 读 取（RxNE=1）。
        /// — 发送时，当一个新数据应该被发送，且数据寄存器还未被写入新的数据（TxE=1）。软件读取 I2C_SR1 寄存器后，对数据寄存器的读或写操作将清除该位；或发送一个起始或停止条件后，或当 PE=0 时，由硬件清除。
        /// 注：在收到一个 NACK 后，BTF 位不会被置位。
        #[inline]
        fn transmit_finish() -> bool {
            Self::block().sr1.read().btf().bit()
        }

        #[inline]
        fn busy() -> bool {
            Self::block().sr2.read().busy().bit()
        }

        #[inline]
        fn clear_pos() {
            Self::block().cr1.modify(|_, w| w.pos().clear_bit());
        }

        fn master_transmit_block(address: u8, buf: &[u8]) -> Result<usize, Error> {
            // 如果总线处于busy状态，则退出
            wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::busy() == false)
                .map_err(|_| Error::Busy)?;

            Self::clear_pos();

            Self::start();
            // SB=1，通过读 SR1，再向 DR 寄存器写数据，实现对该位的清零
            let _ = wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::start_flag())
                .map_err(|_| Error::Start)?;

            Self::transmit(address << 1);
            // ADDR=1，通过读 SR1，再读 SR2，实现对该位的清零
            let _ = wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::address_flag())
                .map_err(|_| Error::Address)?;
            Self::clear_address();

            // TRA 位指示主设备是在接收器模式还是发送器模式。

            let mut iter = buf.iter();
            if let Some(d) = iter.next() {
                // EV8_1：TxE=1, shift 寄存器 empty，数据寄存器 empty，向 DR 寄存器写 Data1
                let _ = wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::tx_empty())
                    .map_err(|_| Error::Tx)?;
                Self::transmit(*d);
            }
            while let Some(t) = iter.next() {
                // EV8：TxE=1, shift 寄存器不 empty，数据寄存器 empty，向 DR 寄存器写 Data2，该位被清零
                let _ = wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::tx_empty())
                    .map_err(|_| Error::Tx)?;
                Self::transmit(*t);
            }
            // EV8_2：TxE=1, BTF=1, 写 Stop 位寄存器，当硬件发出 Stop 位时，TxE 和 BTF 被清零
            let _ = wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::tx_empty())
                .map_err(|_| Error::Tx)?;
            let _ = wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::transmit_finish())
                .map_err(|_| Error::Tx)?;

            Self::stop();

            Ok(buf.len())
        }

        fn master_receive_block(address: u8, buf: &mut [u8]) -> Result<usize, Error> {
            let block = Self::block();
            Self::clear_pos();

            Self::start();
            // EV5：SB=1, 先读 SR1 寄存器，再写 DR 寄存器，清零该位
            let _ = wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::start_flag())
                .map_err(|_| Error::Start)?;

            Self::transmit((address << 1) | 1);

            // EV6：ADDR，先读 SR1，再读 SR2，清零该位
            let _ = wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::address_flag())
                .map_err(|_| Error::Address)?;
            Self::clear_address();

            let len = buf.len();

            let mut enumerate = buf.iter_mut().enumerate();
            while let Some((idx, p)) = enumerate.next() {
                let remain = len - idx;
                if remain > 2 {
                    Self::ack(true);
                    // EV7：RxNE=1, 读 DR 寄存器清零该位
                    let _ = wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::rx_not_empty())
                        .map_err(|_| Error::RX)?;

                    // 读取数据
                    *p = block.dr.read().dr().bits();

                    if Self::transmit_finish() {
                        Self::ack(true);
                        let (_, p) = enumerate.next().unwrap();
                        *p = block.dr.read().dr().bits();
                    }
                } else if remain == 2 {
                    Self::ack(false);

                    // EV7：RxNE=1, 读 DR 寄存器清零该位
                    let _ = wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::rx_not_empty())
                        .map_err(|_| Error::RX)?;
                    // 读取数据
                    *p = block.dr.read().dr().bits();
                    if Self::transmit_finish() {
                        Self::ack(false);
                        let (_, p) = enumerate.next().unwrap();
                        *p = block.dr.read().dr().bits();
                    }
                } else if remain == 1 {
                    Self::ack(false);
                    // EV7：RxNE=1, 读 DR 寄存器清零该位
                    let _ = wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::rx_not_empty())
                        .map_err(|_| Error::RX)?;
                    // 读取数据
                    *p = block.dr.read().dr().bits();
                }
            }

            Self::stop();

            Ok(buf.len())
        }

        fn config(config: Config) -> Result<(), Error> {
            let block = Self::block();
            let plk = sys_pclk();

            Self::enable_config(false);

            // 标准模式下为：2MHz, 快速模式下为：4MHz
            // iic模块时钟，需要使用pclk的hz来匹配
            // 标准模式下 freq >= 4
            // 快速模式下 freq >= 12
            let freq = plk / 1000 / 1000;
            // assert!()
            //freq
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
                // fs bit, false: 标准模式， true：快速模式
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

                // fs bit, false: 标准模式， true：快速模式
                block.ccr.modify(|_, w| w.f_s().bit(true));
                // busy 位只在快速模式下有效
                block.ccr.modify(|_, w| w.duty().bit(duty));
                ccr
            };

            // 设置时钟分频参数
            block.ccr.modify(|_, w| unsafe { w.ccr().bits(ccr as u16) });

            let rise = if config.speed <= SPEED_HZ_STAND {
                freq + 1
            } else {
                freq * 300 / 1000 + 1
            } as u8;
            block.trise.modify(|_, w| unsafe { w.trise().bits(rise) });

            Self::enable_config(true);

            Ok(())
        }
    }
}
