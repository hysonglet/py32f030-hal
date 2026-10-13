//! 32-bit width bit operations

/// Set a contiguous run of bits to 1, all others to 0.
/// BIT_WIDTH: the data width of the operation.
#[inline]
pub fn bit_mask_idx<const BIT_WIDTH: usize>(idx: usize) -> u32 {
    ((0x01u32 << BIT_WIDTH) - 1) << idx
}

/// Modify a contiguous run of bits.
#[inline]
pub fn bit_mask_idx_modify<const BIT_WIDTH: usize>(idx: usize, origin: u32, val: u32) -> u32 {
    assert!(val <= bit_mask_idx::<BIT_WIDTH>(0));

    let mask = bit_mask_idx::<BIT_WIDTH>(idx);
    origin & !mask | (val << idx)
}

/// Clear the specified bit field.
#[inline]
pub fn bit_mask_idx_clear<const BIT_WIDTH: usize>(idx: usize, origin: u32) -> u32 {
    origin & !bit_mask_idx::<BIT_WIDTH>(idx)
}

/// Get the specified bit field.
/// The returned value is already shifted.
#[inline]
pub fn bit_mask_idx_get<const BIT_WIDTH: usize>(idx: usize, origin: u32) -> u32 {
    (bit_mask_idx::<BIT_WIDTH>(idx) & origin) >> idx
}

/// Set the specified bit field to 1.
/// Returns the modified value.
#[inline]
pub fn bit_mask_idx_set<const BIT_WIDTH: usize>(idx: usize, origin: u32) -> u32 {
    origin | bit_mask_idx::<BIT_WIDTH>(idx)
}
