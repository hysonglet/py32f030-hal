#[derive(Debug)]
pub enum Error {
    /// 忙
    Busy,
    /// flash解锁
    Unlock,
    /// flash锁定
    Lock,
    /// 地址未对齐
    Addr,
    /// 超时
    Timeout,
}
