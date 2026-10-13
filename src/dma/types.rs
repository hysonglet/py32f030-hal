use enumset::EnumSetType;

/// Transfer priority.
pub enum Priorities {
    Low = 0,
    Medium = 1,
    High = 2,
    VeryHigh = 3,
}

/// DMA transfer width.
#[derive(Clone, Copy, PartialEq)]
pub enum Burst {
    // 1 byte
    Single = 0,
    // 2 bytes
    Double = 1,
    // 4 bytes
    World = 2,
}

/// DMA mode: one-shot or circular.
#[derive(PartialEq)]
pub enum RepeatMode {
    OneTime(u16),
    Repeat(u16),
}

/// DMA transfer direction.
#[derive(PartialEq)]
pub enum Direction {
    PeriphToMemory,
    MemoryToPeriph,
    MemoryToMemory,
}

#[derive(Debug)]
pub enum Error {
    Busy,
    Address,
    Others,
}

#[derive(EnumSetType)]
pub enum Event {
    GIF,
    TCIF,
    HTIF,
    TEIF,
}
