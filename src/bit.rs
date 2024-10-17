//! 32位宽度的位操作

/// 指定连续的bit位为1，其他为0
#[inline]
pub fn bit_mask_idx<const BIT_WIDTH: usize>(idx: usize) -> u32 {
    ((0x01u32 << BIT_WIDTH) - 1) << idx
}

/// 修改指定连续的bit
#[inline]
pub fn bit_mask_idx_modify<const BIT_WIDTH: usize>(idx: usize, origin: u32, val: u32) -> u32 {
    assert!(val <= bit_mask_idx::<BIT_WIDTH>(0));

    let mask = bit_mask_idx::<BIT_WIDTH>(idx);
    origin & !mask | (val << idx)

    // let val = (val & (bit_mask_idx::<BIT_WIDTH>(0))) << idx;
    // origin & !bit_mask_idx::<BIT_WIDTH>(idx) | val
}

/// 清除指定的bit域
#[inline]
pub fn bit_mask_idx_clear<const BIT_WIDTH: usize>(idx: usize, origin: u32) -> u32 {
    origin & !bit_mask_idx::<BIT_WIDTH>(idx)
}

/// 获取指定的bit域
#[inline]
pub fn bit_mask_idx_get<const BIT_WIDTH: usize>(idx: usize, origin: u32) -> u32 {
    (bit_mask_idx::<BIT_WIDTH>(idx) & origin) >> idx
}

/// 指定bit域修改为1
#[inline]
pub fn bit_mask_idx_set<const BIT_WIDTH: usize>(idx: usize, origin: u32) -> u32 {
    origin | bit_mask_idx::<BIT_WIDTH>(idx)
}
