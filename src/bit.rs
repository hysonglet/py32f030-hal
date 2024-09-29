#[inline]
pub fn bit_mask_idx<const BIT_WIDTH: usize>(idx: usize) -> u32 {
    ((0x01u32 << BIT_WIDTH) - 1) << idx
}

#[inline]
pub fn bit_mask_idx_modify<const BIT_WIDTH: usize>(idx: usize, origin: u32, val: u32) -> u32 {
    let val = (val & (bit_mask_idx::<BIT_WIDTH>(0))) << idx;
    origin & !bit_mask_idx::<BIT_WIDTH>(idx) | val
}

#[inline]
pub fn bit_mask_idx_clear<const BIT_WIDTH: usize>(idx: usize, origin: u32) -> u32 {
    origin & !bit_mask_idx::<BIT_WIDTH>(idx)
}

#[inline]
pub fn bit_mask_idx_get<const BIT_WIDTH: usize>(idx: usize, origin: u32) -> u32 {
    (bit_mask_idx::<BIT_WIDTH>(idx) & origin) >> idx
}

#[inline]
pub fn bit_mask_idx_set<const BIT_WIDTH: usize>(idx: usize, origin: u32) -> u32 {
    origin | bit_mask_idx::<BIT_WIDTH>(idx)
}