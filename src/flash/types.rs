#[derive(Debug)]
pub enum Error {
    /// Busy
    Busy,
    /// Flash unlock
    Unlock,
    /// Flash lock
    Lock,
    /// Address not aligned
    Addr,
    /// Timeout
    Timeout,
}
