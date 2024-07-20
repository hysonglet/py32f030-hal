#![no_std]
#![no_main]

use hal::dma;
use py32f030_hal as hal;

use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());
    defmt::info!("embassy_dma_mem2mem example!");

    let mut src: [u32; 4] = [1, 2, 3, 4];
    let mut dst: [u32; 4] = [0; 4];

    let dma = dma::AnyChannel::new(
        p.DmaChannel1,
        dma::Config::new_mem2mem(
            src.as_mut_ptr() as u32,
            true,
            dst.as_mut_ptr() as u32,
            true,
            dma::Priorities::Low,
            dma::Mode::OneTime(src.len() as u16),
            dma::Burst::World,
        ),
    )
    .unwrap();

    dma.start();
    dma.wait_finish_block();

    defmt::info!("src: {} ", src);
    defmt::info!("dst: {} ", dst);

    loop {
        Timer::after_secs(5).await;
    }
}
