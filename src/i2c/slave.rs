use super::{Error, Event, Instance};
use crate::delay::wait_for_true_timeout_block;
use crate::mode::{Blocking, Mode};
use core::marker::PhantomData;

const WAIT_FLAG_TIMEOUT: usize = 10000;

/// Slave 角色
///
/// 从机流程：
/// 1. 等待主机寻址（`ADDR` 置位），读 SR1 + SR2 清除该标志；
/// 2. 根据 `TRA` 位判断主机要读还是写；
/// 3. 传输结束由 `STOPF`（主机停止）或 `AF`（主机 NACK）标识。
pub struct Slave<'d, T: Instance, M: Mode> {
    _t: PhantomData<(&'d T, M)>,
}

impl<'d, T: Instance, M: Mode> Slave<'d, T, M> {
    pub(super) fn new() -> Self {
        Self { _t: PhantomData }
    }

    /// 总线是否忙
    #[inline]
    pub fn is_busy(&self) -> bool {
        T::busy()
    }

    /// 是否收到广播呼叫地址
    #[inline]
    pub fn is_general_call(&self) -> bool {
        T::is_general_call()
    }
}

impl<'d, T: Instance> Slave<'d, T, Blocking> {
    /// 等待主机寻址本从机，返回主机本次要读（true）还是要写（false）
    ///
    /// 主机写从机 → `TRA=0`；主机读从机 → `TRA=1`
    pub fn wait_addressed(&mut self) -> Result<bool, Error> {
        if wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::ADD)).is_err() {
            return Err(Error::Timeout);
        }

        // 读 SR1 + SR2 清除 ADDR
        T::clear_addr();

        Ok(T::is_transmitter())
    }

    /// 从主机接收数据（主机写、从机收）
    ///
    /// 收到停止条件或缓冲区满即结束，返回实际接收字节数。
    pub fn read_block(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let len = buf.len();
        let mut cnt = 0;
        for v in buf.iter_mut() {
            // 最后一个字节回复 NACK，提示主机停止
            if cnt + 1 == len {
                T::ack(false);
            } else {
                T::ack(true);
            }

            if wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::RXNE))
                .is_err()
            {
                return Err(Error::Timeout);
            }

            *v = T::block().dr.read().dr().bits();
            cnt += 1;

            // 主机提前发出停止条件
            if T::event_flag(Event::STOPF) {
                Self::clear_stop();
                return Ok(cnt);
            }
        }
        Ok(cnt)
    }

    /// 向主机发送数据（主机读、从机发）
    pub fn write_block(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let mut cnt = 0;
        for v in buf.iter() {
            if wait_for_true_timeout_block(WAIT_FLAG_TIMEOUT, || T::event_flag(Event::TXE))
                .is_err()
            {
                return Err(Error::Timeout);
            }

            T::transmit(*v);
            cnt += 1;

            // 主机回复 NACK 表示不再接收
            if T::event_flag(Event::AF) {
                T::event_clear(Event::AF);
                break;
            }

            if T::event_flag(Event::STOPF) {
                Self::clear_stop();
                break;
            }
        }
        Ok(cnt)
    }

    /// 清除停止条件标志（STOPF）：读 SR1 后写 CR1
    fn clear_stop() {
        let _ = T::block().sr1.read().bits();
        T::block().cr1.modify(|_, w| w.pe().set_bit());
    }
}
