//！ MCU时钟配置模块
//! 时钟初始化逻辑，提供获取系统时钟等接口

pub mod peripheral;

use crate::common::Peripheral;
use crate::delay::wait_for_true_timeout_block;
use crate::pac;
use core::marker::PhantomData;

static mut F_CPU: u32 = 8000000;

const TIMEOUT: usize = 10000;

/// 返回系统时钟的频率，单位：Hz
#[inline]
pub fn sys_core_clock() -> u32 {
    unsafe { F_CPU }
}

/// 返回 PCLK 时钟频率，单位： Hz
pub fn sys_pclk() -> u32 {
    let ppre: PclkDiv = Rcc::peripheral().cfgr.read().ppre().bits().into();
    sys_hclk() / ppre.div()
}

/// 返回 HCLK 时钟频率，单位：Hz
pub fn sys_hclk() -> u32 {
    let hpre: HclkDiv = Rcc::peripheral().cfgr.read().hpre().bits().into();
    sys_core_clock() / hpre.div()
}

pub fn timer_pclk() -> u32 {
    let ppre: PclkDiv = Rcc::peripheral().cfgr.read().ppre().bits().into();
    let sys_pclk = sys_pclk();

    if ppre == PclkDiv::Div1 {
        sys_pclk
    } else {
        2 * sys_pclk
    }
}

#[inline]
fn sys_core_clock_update(hz: u32) {
    unsafe {
        F_CPU = hz;
    }
}

/// Rcc 外设
struct Rcc;

impl Peripheral for Rcc {
    type Target = &'static pac::rcc::RegisterBlock;
    fn peripheral() -> Self::Target {
        unsafe { pac::RCC::PTR.as_ref().unwrap() }
    }
}

#[derive(Debug)]
pub enum Error {
    LsiTimeout,
    HseTimeout,
    PllTimeout,
    SysTimeout,
}

/// 返回时钟频率
pub trait ClockFrequency {
    fn hz() -> u32;
}

/// 输入时钟的trait
pub trait Clock {
    #[inline]
    fn enable() -> Result<(), Error> {
        Self::set(true)
    }
    // #[inline]
    // fn disable() -> Result<(), Error> {
    //     Self::set(false)
    // }
    fn set(en: bool) -> Result<(), Error>;
}

/// 低速内部时钟：32KHz
/// Low-speed internal clock
pub struct LSI;

/// 低速外部时钟：32KHz
/// Low-speed external clock
pub struct LSE;

/// 锁相环
/// Phase locked loop
pub struct PLL<CLK: PllSelect> {
    _clk: PhantomData<CLK>,
}

/// HSI 频率选择
#[derive(PartialEq)]
enum HsiHz {
    /// 000: 4MHz
    MHz4 = 0x00,
    /// 001: 8MHz
    MHz8 = 0x01,
    /// 010: 16MHz
    MHz16 = 0x03,
    /// 011: 22.12Mhz
    MHz22_12 = 0x04,
    /// 100: 24MHz
    MHz24 = 0x05,
}

impl From<u8> for HsiHz {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::MHz4,
            1 => Self::MHz8,
            2 => Self::MHz16,
            3 => Self::MHz22_12,
            4 => Self::MHz16,
            5 => Self::MHz22_12,
            6 => Self::MHz24,
            _ => Self::MHz4,
        }
    }
}

impl HsiHz {
    fn hz(&self) -> u32 {
        match *self {
            Self::MHz4 => 4000000,
            Self::MHz8 => 8000000,
            Self::MHz16 => 16000000,
            Self::MHz22_12 => 22120000,
            Self::MHz24 => 24000000,
        }
    }
}

/// 高速内部时钟
/// High-speed internal clock
pub struct HSI;

/// HSE 高速外部时钟 （4~32M）
/// High-speed external clock
pub struct HSE<const HZ: u32 = 24000000>;

impl ClockFrequency for LSI {
    fn hz() -> u32 {
        32768
    }
}

impl ClockFrequency for LSE {
    fn hz() -> u32 {
        32768
    }
}

impl ClockFrequency for HSI {
    fn hz() -> u32 {
        let peripheral = Rcc::peripheral();
        let hsi_fs: HsiHz = peripheral.icscr.read().hsi_fs().bits().into();
        hsi_fs.hz()
    }
}

impl<const HZ: u32> ClockFrequency for HSE<HZ> {
    fn hz() -> u32 {
        HZ
    }
}

impl<CLK> ClockFrequency for PLL<CLK>
where
    CLK: PllSelect + ClockFrequency,
{
    fn hz() -> u32 {
        CLK::hz() * 2
    }
}

impl Clock for LSI {
    #[inline]
    fn set(en: bool) -> Result<(), Error> {
        // Rcc::peripheral()
        //     .bdcr
        //     .modify(|_, w| w.lscoen().bit(en).lscosel().bit(en));
        // Rcc::peripheral()
        //     .bdcr
        //     .modify(|_, w| w.lscosel().clear_bit().lscoen().set_bit());
        let block = Rcc::peripheral();
        block.csr.modify(|_, w| w.lsion().bit(en));
        wait_for_true_timeout_block(1000, || block.csr.read().lsirdy() == en)
            .map_err(|_| Error::LsiTimeout)
    }
}

impl Clock for LSE {
    #[inline]
    fn set(en: bool) -> Result<(), Error> {
        let peripheral = Rcc::peripheral();

        peripheral
            .bdcr
            .modify(|_, w| w.lscosel().set_bit().lscoen().clear_bit());

        // 00：关闭 LSE；
        // 01：弱驱动能力；（默认）
        // 10：中驱动能力；（推荐）
        // 11：强驱动能力；
        let lse_driver = if en { 0b01 } else { 0 };

        peripheral
            .ecscr
            .modify(|_, w| unsafe { w.lse_driver().bits(lse_driver) });

        Ok(())
    }
}

impl Clock for HSI {
    #[inline]
    fn set(en: bool) -> Result<(), Error> {
        let peripheral = Rcc::peripheral();
        peripheral.cr.modify(|_, w| w.hsion().bit(en));
        Ok(())
    }
}

impl<const HZ: u32> Clock for HSE<HZ> {
    #[inline]
    fn set(en: bool) -> Result<(), Error> {
        let peripheral = Rcc::peripheral();

        peripheral.cr.modify(|_, w| w.hseon().bit(en));

        assert!(
            HZ < 4000000 || HZ > 32000000,
            "HZ:{} only allow in [4~32M]",
            HZ
        );
        // if HZ < 4000000 || HZ > 32000000 {
        //     panic!("HZ only allow in [4~32M]");
        // }

        let v = if HZ < 8000000 {
            1
        } else if HZ < 16000000 {
            2
        } else {
            3
        };
        peripheral
            .ecscr
            .modify(|_, w| unsafe { w.hse_freq().bits(v) });

        let mut timeout = TIMEOUT;
        while peripheral.cr.read().hserdy().bit_is_clear() {
            cortex_m::asm::delay(1000);
            timeout -= 1;
            if timeout == 0 {
                return Err(Error::HseTimeout);
            }
        }

        Ok(())
    }
}

impl<CLK> Clock for PLL<CLK>
where
    CLK: PllSelect,
{
    fn set(en: bool) -> Result<(), Error> {
        CLK::set(en)
    }
}

enum HsiDiv {
    DIV1 = 0,
    DIV2 = 1,
    DIV4 = 2,
    DIV8 = 3,
    DIV16 = 4,
    DIV32 = 5,
    DIV64 = 6,
    DIV128 = 7,
}

impl From<u32> for HsiDiv {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::DIV1,
            2 => Self::DIV2,
            4 => Self::DIV4,
            8 => Self::DIV8,
            16 => Self::DIV16,
            32 => Self::DIV32,
            64 => Self::DIV64,
            128 => Self::DIV128,
            _ => unreachable!(), // _ => panic!("HSI DIV only allowd in [1, 2, 4, 8, 32, 64, 128]"),
        }
    }
}

/// DIV = [1, 2, 4, 8, 16, 32, 64, 128]
pub struct HSIDiv<const DIV: u32 = 0x00> {
    // _hsi: PhantomData<HSI>,
}

impl<const DIV: u32> ClockFrequency for HSIDiv<DIV> {
    fn hz() -> u32 {
        HSI::hz() / DIV
    }
}

impl<const DIV: u32> Clock for HSIDiv<DIV> {
    fn set(en: bool) -> Result<(), Error> {
        let peripheral = Rcc::peripheral();

        let hsi_div: HsiDiv = DIV.into();
        // 设置分频
        peripheral
            .cr
            .modify(|_, w| unsafe { w.hsidiv().bits(hsi_div as u8) });

        HSI::set(en)
    }
}

#[derive(PartialEq)]
enum PllClock {
    Hsi = 0,
    Hse = 1,
}

impl PllClock {
    fn config(&self) -> Result<(), Error> {
        let peripheral = Rcc::peripheral();

        peripheral
            .pllcfgr
            .modify(|_, w| w.pllsrc().bit(*self == PllClock::Hse));

        peripheral.cr.modify(|_, w| w.pllon().set_bit());

        wait_for_true_timeout_block(TIMEOUT, || peripheral.cr.read().pllrdy().bit())
            .map_err(|_| Error::PllTimeout)?;
        Ok(())
    }
}

/// PLL 可以用来对 HSI 或者 HSE 进行倍频。在使能 PLL 之前，必须对 PLL 进行配置。一旦 PLL 被使能，这 些被配置的寄存器不能被改变。
pub trait PllSelect: Clock {
    fn config() -> Result<(), Error>;
}

impl PllSelect for HSI {
    fn config() -> Result<(), Error> {
        Self::enable()?;

        PllClock::Hsi.config()
    }
}

impl<const HZ: u32> PllSelect for HSE<HZ> {
    fn config() -> Result<(), Error> {
        Self::enable()?;
        PllClock::Hse.config()?;
        Ok(())
    }
}

/// Sysclk 选择
#[derive(Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
enum SysClockSw {
    HSISys = 0,
    HSE = 1,
    PLL = 2,
    LSI = 3,
    LSE = 4,
}

impl SysClockSw {
    fn config(&self) -> Result<(), Error> {
        let peripheral = Rcc::peripheral();

        peripheral
            .cfgr
            .modify(|_, w| unsafe { w.sw().bits(*self as u8) });
        wait_for_true_timeout_block(TIMEOUT, || {
            peripheral.cfgr.read().sws().bits() == peripheral.cfgr.read().sw().bits()
        })
        .map_err(|_| Error::SysTimeout)?;
        Ok(())
    }
}

pub trait SysClkSelect: Clock + ClockFrequency {
    fn config() -> Result<(), Error>;
}

impl<const DIV: u32> SysClkSelect for HSIDiv<DIV> {
    fn config() -> Result<(), Error> {
        // 开启时钟
        Self::enable()?;

        SysClockSw::HSISys.config()?;
        sys_core_clock_update(Self::hz());
        Ok(())
    }
}

impl<const HZ: u32> SysClkSelect for HSE<HZ> {
    fn config() -> Result<(), Error> {
        Self::enable()?;

        SysClockSw::HSE.config()?;
        sys_core_clock_update(Self::hz());
        Ok(())
    }
}

impl SysClkSelect for LSI {
    fn config() -> Result<(), Error> {
        Self::enable()?;
        SysClockSw::LSI.config()?;
        sys_core_clock_update(Self::hz());
        Ok(())
    }
}

impl SysClkSelect for LSE {
    fn config() -> Result<(), Error> {
        Self::enable()?;
        // 更新时钟
        SysClockSw::LSE.config()?;
        sys_core_clock_update(Self::hz());
        Ok(())
    }
}

impl<CLK> SysClkSelect for PLL<CLK>
where
    CLK: PllSelect + ClockFrequency,
{
    fn config() -> Result<(), Error> {
        CLK::config()?;

        SysClockSw::PLL.config()?;

        // while true {}
        sys_core_clock_update(Self::hz());

        Ok(())
    }
}

/// PLK 时钟分频
#[derive(Clone, Copy, PartialEq)]
pub enum PclkDiv {
    Div1 = 1,
    Div2 = 4,
    Div4 = 5,
    Div8 = 6,
    Div16 = 7,
}

/// Hcl 时钟分频
#[derive(Clone, Copy)]
pub enum HclkDiv {
    Div1 = 1,
    Div2 = 8,
    Div4 = 9,
    Div8 = 10,
    Div16 = 11,
    Div64 = 12,
    Div128 = 13,
    Div256 = 14,
    Div512 = 15,
}

impl From<u8> for PclkDiv {
    fn from(value: u8) -> Self {
        match value {
            0..=3 => Self::Div1,
            4 => Self::Div2,
            5 => Self::Div4,
            6 => Self::Div8,
            7 => Self::Div16,
            _ => panic!("Error"),
        }
    }
}

impl From<u8> for HclkDiv {
    fn from(value: u8) -> Self {
        match value {
            0..=7 => Self::Div1,
            8 => Self::Div2,
            9 => Self::Div4,
            10 => Self::Div8,
            11 => Self::Div16,
            12 => Self::Div64,
            13 => Self::Div128,
            14 => Self::Div256,
            15 => Self::Div512,
            _ => panic!("Error"),
        }
    }
}

impl HclkDiv {
    fn div(&self) -> u32 {
        match *self {
            Self::Div1 => 1,
            Self::Div2 => 2,
            Self::Div4 => 4,
            Self::Div8 => 8,
            Self::Div16 => 16,
            Self::Div64 => 64,
            Self::Div128 => 128,
            Self::Div256 => 256,
            Self::Div512 => 512,
        }
    }
    pub fn config(&self) {
        let peripheral = Rcc::peripheral();
        peripheral
            .cfgr
            .modify(|_, w| unsafe { w.hpre().bits(*self as u8) });
    }
}

impl PclkDiv {
    fn div(&self) -> u32 {
        match *self {
            Self::Div1 => 1,
            Self::Div2 => 2,
            Self::Div4 => 4,
            Self::Div8 => 8,
            Self::Div16 => 16,
        }
    }

    pub fn config(&self) {
        let peripheral = Rcc::peripheral();
        peripheral
            .cfgr
            .modify(|_, w| unsafe { w.ppre().bits(*self as u8) });
    }
}

/// Mco输出分频
pub enum McoDIV {
    DIV1 = 0,
    DIV2 = 1,
    DIV4 = 2,
    DIV8 = 3,
    DIV16 = 4,
    DIV32 = 5,
    DIV64 = 6,
    DIV128 = 7,
}

impl From<u32> for McoDIV {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::DIV1,
            2 => Self::DIV2,
            4 => Self::DIV4,
            8 => Self::DIV8,
            16 => Self::DIV16,
            32 => Self::DIV32,
            64 => Self::DIV64,
            128 => Self::DIV128,
            _ => unreachable!(),
            // _ => panic!("MCO DIV only allowd in [1, 2, 4, 8, 32, 64, 128]"),
        }
    }
}

/// 系统时钟
pub struct SysClock<CLK: SysClkSelect> {
    _clk: PhantomData<CLK>,
}

impl<CLK: SysClkSelect> SysClock<CLK> {
    pub fn config() -> Result<Self, Error> {
        CLK::config()?;
        Ok(Self { _clk: PhantomData })
    }

    pub fn hz(&self) -> u32 {
        CLK::hz()
    }
}

/// MCO output
pub struct Mco;

/// MCO 输出选择器
pub enum McoSelect {
    Disable = 0,
    SysClk = 1,
    Hsi = 3,
    Hse = 4,
    Pll = 5,
    Lsi = 6,
    Lse = 7,
}

impl Mco {
    pub fn select(source: McoSelect, div: McoDIV) {
        let peripheral = Rcc::peripheral();

        peripheral
            .cfgr
            .modify(|_, w| unsafe { w.mcopre().bits(div as u8).mcosel().bits(source as u8) })
    }
}

/// RTC 时钟选择器
pub trait RtcSelect: Clock + ClockFrequency {
    fn config() -> Result<(), Error>;
}

impl RtcSelect for LSI {
    fn config() -> Result<(), Error> {
        Self::enable()?;
        Rcc::peripheral()
            .bdcr
            .modify(|_, w| unsafe { w.rtcsel().bits(0b10).rtcen().set_bit().lscoen().set_bit() });
        Ok(())
    }
}
impl RtcSelect for LSE {
    fn config() -> Result<(), Error> {
        // 开启外部时钟，
        // TODO！ 引脚初始化？
        Self::enable()?;
        Rcc::peripheral()
            .bdcr
            .modify(|_, w| unsafe { w.rtcsel().bits(0b10).rtcen().set_bit().lscoen().set_bit() });
        Ok(())
    }
}
impl<const HZ: u32> RtcSelect for HSE<HZ> {
    fn config() -> Result<(), Error> {
        Self::enable()?;
        Rcc::peripheral()
            .bdcr
            .modify(|_, w| unsafe { w.rtcsel().bits(0b11).rtcen().set_bit().lscoen().set_bit() });
        Ok(())
    }
}

pub(crate) struct RtcClock<CLK: RtcSelect> {
    _clk: PhantomData<CLK>,
}

impl<CLK: RtcSelect> RtcClock<CLK> {
    pub(crate) fn config() -> Result<(), Error> {
        CLK::config()
    }
}

impl ClockFrequency for RtcClock<LSE> {
    fn hz() -> u32 {
        LSE::hz()
    }
}

impl ClockFrequency for RtcClock<LSI> {
    fn hz() -> u32 {
        LSI::hz()
    }
}

impl<const HZ: u32> ClockFrequency for RtcClock<HSE<HZ>> {
    fn hz() -> u32 {
        HSE::<HZ>::hz() / 128
    }
}

pub(crate) fn lsi_enable() -> Result<(), Error> {
    LSI::enable()
}
