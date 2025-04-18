mod hal;
mod types;

use crate::delay::wait_for_true_timeout_block;
use core::marker::PhantomData;
use drop_move::DropGuard;
use embassy_hal_internal::{into_ref, Peripheral};

use types::*;

pub const FLASH_PAGE_SIZE: usize = 128;
pub const FLASH_PAGE_PER_SECTOR_CNT: usize = FLASH_PAGE_SIZE / 4;
pub const FLASH_SECTOR_SIZE: usize = 4 * 1024;
pub const FLASH_SECTOR_CNT: usize = 16;
pub const MAIN_FLASH_SIZE: usize = FLASH_SECTOR_SIZE * FLASH_SECTOR_CNT;
pub const FLASH_BASE_ADDR: u32 = 0x08000000;
pub const FLASH_END_ADDR: u32 = FLASH_BASE_ADDR + MAIN_FLASH_SIZE as u32;

pub(crate) const WAIT_TICK_TIMEOUT: usize = 100000;

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}
impl Instance for crate::mcu::peripherals::FLASH {}
impl hal::sealed::Instance for crate::mcu::peripherals::FLASH {}

pub struct Flash<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> Flash<'d, T> {
    pub fn new(_flash: impl Peripheral<P = T>) -> Self {
        into_ref!(_flash);

        Self { _t: PhantomData }
    }

    pub fn uuid() -> [u8; 16] {
        T::uuid()
    }

    pub fn erase_flash(&self) -> Result<(), Error> {
        T::unlock()?;

        let _drop = DropGuard::new(|| T::lock().unwrap());

        T::mass_erase();

        // 等待擦除完毕
        wait_for_true_timeout_block(WAIT_TICK_TIMEOUT, || !T::busy())
            .map_err(|_| Error::Timeout)?;

        T::lock()
    }

    /// page 擦除
    pub fn erase_page(&self, addr: u32) -> Result<(), Error> {
        T::unlock()?;

        let _drop = DropGuard::new(|| T::lock().unwrap());

        // 地址错误
        if !(FLASH_BASE_ADDR..FLASH_END_ADDR).contains(&addr) || addr % FLASH_PAGE_SIZE as u32 != 0
        {
            return Err(Error::Addr);
        }

        T::page_erase(addr);

        // 等待擦除完毕
        wait_for_true_timeout_block(WAIT_TICK_TIMEOUT, || !T::busy())
            .map_err(|_| Error::Timeout)?;

        T::lock()
    }

    ///  给定扇区和页编号，自动计算出页的地址，然后擦除
    pub fn erase_page_by_index(&self, sector: usize, page: usize) -> Result<(), Error> {
        let addr = FLASH_BASE_ADDR
            + sector as u32 * FLASH_SECTOR_SIZE as u32
            + page as u32 * FLASH_PAGE_SIZE as u32;
        self.erase_page(addr)
    }

    pub fn erase_sector(&self, addr: u32) -> Result<(), Error> {
        T::unlock()?;

        let _drop = DropGuard::new(|| T::lock().unwrap());

        // 地址错误
        if !(FLASH_BASE_ADDR..FLASH_END_ADDR).contains(&addr)
            || addr % FLASH_SECTOR_SIZE as u32 != 0
        {
            return Err(Error::Addr);
        }

        T::sector_erase(addr);

        // 等待擦除完毕
        wait_for_true_timeout_block(WAIT_TICK_TIMEOUT, || !T::busy())
            .map_err(|_| Error::Timeout)?;

        T::lock()
    }

    pub fn erase_sector_by_index(&self, sector: usize) -> Result<(), Error> {
        let addr = FLASH_BASE_ADDR + sector as u32 * FLASH_SECTOR_SIZE as u32;
        self.erase_page(addr)
    }

    pub fn program_page(
        &self,
        page_addr: u32,
        content: [u32; FLASH_PAGE_SIZE / 4],
    ) -> Result<(), Error> {
        // 地址错误
        if !(FLASH_BASE_ADDR..FLASH_END_ADDR).contains(&page_addr)
            || page_addr % FLASH_PAGE_SIZE as u32 != 0
        {
            return Err(Error::Addr);
        }

        T::unlock()?;

        T::page_program(page_addr, content);

        T::lock()
    }

    /// 读取一个 page 内容
    pub fn read_page(&self, addr: u32, buf: &mut [u32; FLASH_PAGE_SIZE / 4]) -> Result<(), Error> {
        // 地址错误
        if !(FLASH_BASE_ADDR..FLASH_END_ADDR).contains(&addr) || addr % FLASH_PAGE_SIZE as u32 != 0
        {
            return Err(Error::Addr);
        }

        for (i, item) in buf.iter_mut().enumerate() {
            let addr = addr + i as u32 * 4;
            *item = unsafe { core::ptr::read_volatile(addr as *const u32) };
        }

        Ok(())
    }

    /// 返回一个 page 内容
    pub fn read_page_by_index(
        &self,
        sector: usize,
        page: usize,
        buf: &mut [u32; FLASH_PAGE_SIZE / 4],
    ) -> Result<(), Error> {
        let addr = FLASH_BASE_ADDR
            + sector as u32 * FLASH_SECTOR_SIZE as u32
            + page as u32 * FLASH_PAGE_SIZE as u32;

        self.read_page(addr, buf)
    }

    /// 返回页地址
    pub fn get_page_addr(sector: usize, page: usize) -> Result<u32, Error> {
        if sector >= FLASH_SECTOR_CNT || page >= FLASH_PAGE_PER_SECTOR_CNT {
            return Err(Error::Addr);
        }
        let addr = FLASH_BASE_ADDR
            + sector as u32 * FLASH_SECTOR_SIZE as u32
            + page as u32 * FLASH_PAGE_SIZE as u32;

        Ok(addr)
    }

    // pub fn write_page(&self, addr: u32, buf: &[u32; FLASH_PAGE_SIZE / 4]) -> Result<(), Error> {}
}
