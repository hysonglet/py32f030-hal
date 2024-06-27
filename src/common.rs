pub trait PeriphPtr {
    type Target;

    fn block() -> Self::Target;
}

pub enum Error {
    Timeout,
}

pub fn wait_fun<F>(tick: u32, f: F) -> Result<(), Error>
where
    F: Fn() -> bool,
{
    let mut cnt = tick;
    #[allow(clippy::never_loop)]
    loop {
        if f() {
            return Ok(());
        }
        cnt -= 1;
        cortex_m::asm::delay(1);
        if cnt == 0 {
            return Err(Error::Timeout);
        }
    }
}

pub(crate) trait Peripheral {
    type Target;
    fn peripheral() -> Self::Target;
}

pub(crate) struct BitOption;
impl BitOption {
    #[inline]
    pub fn bit_mask_idx<const BIT_WIDTH: usize>(idx: usize) -> u32 {
        ((0x01u32 << BIT_WIDTH) - 1) << (BIT_WIDTH * idx)
    }
    #[inline]
    pub(crate) fn bit_mask_idx_modify<const BIT_WIDTH: usize>(
        pin: usize,
        origin: u32,
        val: u32,
    ) -> u32 {
        let val = val << (BIT_WIDTH * pin);
        origin & !Self::bit_mask_idx::<BIT_WIDTH>(pin) | val
    }
    #[inline]
    pub(crate) fn bit_mask_idx_clear<const BIT_WIDTH: usize>(idx: usize, origin: u32) -> u32 {
        origin & !Self::bit_mask_idx::<BIT_WIDTH>(idx)
    }
    #[inline]
    pub(crate) fn bit_mask_idx_get<const BIT_WIDTH: usize>(idx: usize, origin: u32) -> u32 {
        (Self::bit_mask_idx::<BIT_WIDTH>(idx) & origin) >> (BIT_WIDTH * idx)
    }

    #[inline]
    pub(crate) fn bit_mask_idx_set<const BIT_WIDTH: usize>(idx: usize, origin: u32) -> u32 {
        origin | Self::bit_mask_idx::<BIT_WIDTH>(idx)
    }
}
