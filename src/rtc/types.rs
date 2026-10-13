use enumset::EnumSetType;

#[derive(Debug)]
pub enum Error {
    Clock,
    Timeout,
}

/// Alarm or second output selection
// pub enum PinSignal {
//     /// Alarm signal output on the pin
//     AlarmPulse,
//     /// Second signal output on the pin
//     SecondPulse,
//     /// RTC clock signal output on the pin
//     Clock,
// }

#[derive(PartialEq)]
pub enum RtcClock {
    LSI,
    LSE,
    HSE_DIV_32,
}

#[derive(EnumSetType, Debug)]
pub enum EventKind {
    Alarm,
    Second,
    OverFlow,
}
