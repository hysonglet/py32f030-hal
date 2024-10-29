pub(super) mod sealed {
    use super::super::*;
    use crate::clock::sys_pclk;
    use crate::delay::wait_for_true_timeout_block;
    use crate::i2c::Id;
    use crate::pac;
    pub const WAIT_FLAG_TIMEOUT: usize = 100_000;

    // 总线标志等待超时， 100 ms

    pub trait Instance {
        // 考虑以后其他单片机可能有多个IIC
        fn id() -> Id;

        #[inline]
        fn block() -> &'static pac::i2c::RegisterBlock {
            match Self::id() {
                Id::I2c1 => unsafe { pac::I2C::PTR.as_ref().unwrap() },
            }
        }

        /// 使能配置
        fn enable_config(en: bool) {
            Self::block().cr1.modify(|_, w| w.pe().bit(en))
        }

        /// 重启 I2c外设
        #[inline]
        fn reset() {
            Self::id().clock().reset()
        }

        /// 查看总线是否释放
        #[inline]
        fn is_bus_release() -> bool {
            Self::block().cr1.read().swrst().bit()
        }

        /// 生成开始信号
        #[inline]
        fn start() {
            Self::block().cr1.modify(|_, w| w.start().set_bit())
        }

        /// 生成停止信号
        #[inline]
        fn stop() {
            Self::block().cr1.modify(|_, w| w.stop().set_bit())
        }

        /// 软重启
        /// 当被置位时，I2C 处于复位状态。在复位释放前，要确保 I2C 的引脚被释放，总线是空闲状态。
        /// - 0：I2C 模块不处于复位状态
        ///
        /// - 1：I2C 模块处于复位状态
        ///   注：该位可以用于 error 或 locked 状态时重新初始化 I2C。如 BUSY 位为 1，在总线上又没有检测到停止条件时。
        #[inline]
        fn soft_reset() {
            Self::block().cr1.modify(|_, w| w.swrst().set_bit())
        }

        /// 将数据写入传输寄存器
        #[inline]
        fn transmit(data: u8) {
            Self::block().dr.modify(|_, w| unsafe { w.dr().bits(data) });
        }

        /// 设置回复ack或nack
        #[inline]
        fn ack(is_ack: bool) {
            Self::block().cr1.modify(|_, w| w.ack().bit(is_ack))
        }

        /// 写入从地址寄存器
        #[inline]
        fn address(address: u8) {
            Self::block()
                .oar1
                .modify(|_, w| unsafe { w.add().bits(address) });
        }

        /// 总线忙
        #[inline]
        fn busy() -> bool {
            Self::block().sr2.read().busy().bit()
        }

        /// 开启或关闭中断事件
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

        /// 返回是否匹配到中断事件了，
        /// 如果开启了事件中断并且存在事件标志，则返回true
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
                    //软件读取 I2C_SR1 寄存器后，对数据寄存器的写操作将清除该位； 或当 PE=0 时，由硬件清除
                    w
                }
                Event::ADD => {
                    //软件读取 I2C_SR1 寄存器后，再读 I2C_SR2 寄存器将清除该位；当 PE=0 时，由硬件清除。
                    let r = r.bits();
                    let _ = Self::block().sr2.read();
                    unsafe { w.bits(r) }
                }
                Event::STOPF => {
                    // 软件读取 I2C_SR1 寄存器后，对 I2C_CR1 寄存器的写操作将清除该位，或当 PE=0 时，硬件清除该位。
                    w
                }
                Event::BTF => {
                    // 软件读取 I2C_SR1 寄存器后，对数据寄存器的读或写操作将清除该位；或发送一个起始或停止条件后，或当 PE=0 时，由硬件清除。
                    w
                }
                Event::RXNE => {
                    // 在接收时，当数据寄存器不为空，置位该寄存器。在接收地址阶段，该寄存器不置位。软件对数据寄存器的读写操作会清除该寄存器，
                    // 或当 PE=0 时由硬件清除。
                    // 注：当设置了 BTF 时，读取数据不能清除 RxNE
                    // 位，因为此时数据寄存器仍为满。
                    w
                }
                Event::TXE => {
                    //软件写数据到 DR 寄存器可清除该位，或在发生一个起始或停止条件后，或当 PE=0 时由硬件自动清除。
                    w
                }
                Event::BERR => w.berr().clear_bit(),
                Event::ARLO => w.arlo().clear_bit(),
                Event::AF => w.af().clear_bit(),
                Event::OVR => w.ovr().clear_bit(),
                Event::PECERR => w.pecerr().clear_bit(),
            });
        }

        /// ACK/PEC 位置（用于数据接收），软件可置位/清零该寄存器，或 PE=0 时由硬件清零。
        ///
        /// - 0：ACK 位控制当前移位寄存器内正在接收的字节的(N)ACK。PEC 位表明当前移位寄存器内的字节是 PEC
        /// - 1：ACK 位控制在移位寄存器里接收的下一个字节的(N)ACK。PEC 位表明在移位寄存器里接收的下一个字节是 PEC
        ///      注：POS 位只能用在 2 字节的接收配置中，必须在接收数据之前配置。为了 NACK 第 2 个字节，必须在清除 ADDR 之后清除 ACK 位。为了检测第 2 个字节的 PEC，必须在配置了POS 位之后，ADDR stretch 事件时设置 PEC位。
        #[inline]
        fn clear_pos() {
            Self::block().cr1.modify(|_, w| w.pos().clear_bit());
        }

        fn master_transmit_block(address: u8, buf: &[u8]) -> Result<usize, Error> {
            // 如果总线处于busy状态，则退出
            // wait_for_flag_timeout(WAIT_FLAG_TIMEOUT_US, || Self::busy() == false)
            //     .map_err(|_| Error::Busy)?;

            Self::clear_pos();

            Self::start();
            // SB=1，通过读 SR1，再向 DR 寄存器写数据，实现对该位的清零
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::SB))
                .map_err(|_| {
                    Self::event_clear(Event::ARLO);
                    Error::Start
                })?;

            Self::transmit(address << 1);

            // ADDR=1，通过读 SR1，再读 SR2，实现对该位的清零
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::ADD))
                .map_err(|_| {
                    // Self::debug();
                    // 清除 af 置位
                    Self::event_clear(Event::AF);
                    Self::stop();
                    Error::Address
                })?;
            Self::event_clear(Event::ADD);

            // TRA 位指示主设备是在接收器模式还是发送器模式。

            let mut iter = buf.iter();
            if let Some(d) = iter.next() {
                // EV8_1：TxE=1, shift 寄存器 empty，数据寄存器 empty，向 DR 寄存器写 Data1
                wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::TXE))
                    .map_err(|_| Error::Tx)?;
                Self::transmit(*d);
            }

            // 接着将后面的数据发送出去
            for t in iter {
                // EV8：TxE=1, shift 寄存器不 empty，数据寄存器 empty，向 DR 寄存器写 Data2，该位被清零
                wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::TXE))
                    .map_err(|_| {
                        Self::stop();
                        Error::Tx
                    })?;
                Self::transmit(*t);
            }

            // EV8_2：TxE=1, BTF=1, 写 Stop 位寄存器，当硬件发出 Stop 位时，TxE 和 BTF 被清零
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
            // EV5：SB=1, 先读 SR1 寄存器，再写 DR 寄存器，清零该位
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::SB))
                .map_err(|_| Error::Start)?;

            Self::transmit((address << 1) | 1);

            // EV6：ADDR，先读 SR1，再读 SR2，清零该位
            wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || Self::event_flag(Event::ADD))
                .map_err(|_| Error::Address)?;
            Self::event_clear(Event::ADD);

            let len = buf.len();

            let mut enumerate = buf.iter_mut().enumerate();
            while let Some((idx, p)) = enumerate.next() {
                let remain = len - idx;
                if remain > 2 {
                    Self::ack(true);
                    // EV7：RxNE=1, 读 DR 寄存器清零该位
                    wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || {
                        Self::event_flag(Event::RXNE)
                    })
                    .map_err(|_| Error::RX)?;

                    // 读取数据
                    *p = block.dr.read().dr().bits();

                    if Self::event_flag(Event::BTF) {
                        Self::ack(true);
                        let (_, p) = enumerate.next().unwrap();
                        *p = block.dr.read().dr().bits();
                    }
                } else if remain == 2 {
                    Self::ack(false);

                    // EV7：RxNE=1, 读 DR 寄存器清零该位
                    wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || {
                        Self::event_flag(Event::RXNE)
                    })
                    .map_err(|_| Error::RX)?;
                    // 读取数据
                    *p = block.dr.read().dr().bits();
                    if Self::event_flag(Event::BTF) {
                        Self::ack(false);
                        let (_, p) = enumerate.next().unwrap();
                        *p = block.dr.read().dr().bits();
                    }
                } else if remain == 1 {
                    Self::ack(false);
                    // EV7：RxNE=1, 读 DR 寄存器清零该位
                    wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || {
                        Self::event_flag(Event::RXNE)
                    })
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

            Self::enable_config(false);

            let plk = sys_pclk();
            // 标准模式下为：2MHz, 快速模式下为：4MHz
            // iic模块时钟，需要使用pclk的hz来匹配
            // 标准模式下 freq >= 4
            // 快速模式下 freq >= 12
            let freq = plk / 1000 / 1000;

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
