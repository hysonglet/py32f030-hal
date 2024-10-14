#[derive(Debug)]
pub enum Error {
    Clock,
    Timeout,
}

/// Alarm or second output selection
pub enum PinSignal {
    /// Pin 上输出的是 alarm 信号
    AlarmPulse,
    /// Pin 上输出的是秒信号
    SecondPulse,
    /// Pin 上输出的是RTC clock 信号
    Clock,
}

#[derive(PartialEq)]
pub enum RtcClock {
    LSI,
    LSE,
    HSE_DIV_32,
}
