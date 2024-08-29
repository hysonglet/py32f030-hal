#![no_std]
#![no_main]

use hal::common::BitOption;
use py32f030_hal as hal;
use {defmt_rtt as _, panic_probe as _};
#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("{:b}", 0b110);
    defmt::info!("{:b}", BitOption::bit_mask_idx::<3>(3));
    defmt::info!(
        "{:b}",
        BitOption::bit_mask_idx_modify::<3>(3, 0b110110, 0b1011)
    );
    defmt::info!("{:b}", BitOption::bit_mask_idx_clear::<3>(1, 0b110110));
    defmt::info!("{:b}", BitOption::bit_mask_idx_get::<3>(1, 0b110100));
    defmt::info!("{:b}", BitOption::bit_mask_idx_set::<3>(2, 0b110100));
    loop {
        cortex_m::asm::wfe();
    }
}
