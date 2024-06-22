
use crate::common::PeriphPtr;
use crate::pac;
use crate::common;

// const CRYSTAL_HZ: u32 = lazy_static::lazy_static!{
//     core::env!("CRYSTAL_HZ").
// };

pub trait CLK_HZ {
    fn hz(&self) -> u32;
    fn clk() -> u32;
}

pub trait Config {
    fn config(en: bool) -> Result<(), ()>;
}

/// MCU 的时钟有四种来源，可选择外部高速或低速、内部高速或低速,
///  - HSI       （4~24MHz）
///  - LSI        32.768kHz）
///  - HSE
///    - 数字时钟
///    - 晶振    （4-32MHz）
///  - LSE      （32.768kHz）
pub mod clk_select{
    use crate::common::PeriphPtr;
    use super::CLK_HZ;

    pub struct CLK_LSE;
    
    #[derive(PartialEq, Eq)]
    #[derive(Copy, Clone)]
    pub enum HsiClk {
        Hz4M = 0b000,
        Hz8M = 0b001,
        Hz16M = 0b010,
        Hz22_12M = 0b011,
        Hz24M = 0b100,
    }

    #[derive(PartialEq, Eq, Clone, Copy)]
    pub enum HsiDiv{
        HsiDiv1 = 0b000,
        HsiDiv2 = 0b001,
        HsiDiv4 = 0b010,
        HsiDiv8 = 0b011,
        HsiDiv16 = 0b100,
        HsiDiv32 = 0b101,
        HsiDiv64 = 0b110,
        HsiDiv128 = 0b111,
    }

    impl CLK_HZ for HsiClk {
        fn hz(&self) -> u32 {
            match *self {
                HsiClk::Hz4M => 4*1000*1000,
                HsiClk::Hz8M => 8*1000*1000,
                HsiClk::Hz16M => 16*1000*1000,
                HsiClk::Hz22_12M => 221200*1000,
                HsiClk::Hz24M => 24*1000*1000
            }
        }
        fn clk() -> u32 {
            unimplemented!()
        }
    }

    impl HsiDiv {
        pub(super) fn config(&self){
            let rb = super::Rcc::block();
            unsafe {
                rb.cr.write(|w| w.hsidiv().bits(*self as u8))
            }
        }
    }

    impl Default for HsiClk {
        fn default() -> Self {
            HsiClk::Hz8M
        }
    }

    impl Default for HsiDiv {
        fn default() -> Self {
            HsiDiv::HsiDiv1
        }
    }

    impl HsiClk {
        fn trim(&self) -> u32 {
            const HSI_TRIM_ADDR: [u32; 5] = [0x1FFF_0F00, 0x1FFF_0F04, 0x1FFF_0F08, 0x1FFF_0F0C, 0x1FFF_0F10];
            unsafe {
                match *self {
                    HsiClk::Hz4M => core::ptr::read(HSI_TRIM_ADDR[0] as *const u32),
                    HsiClk::Hz8M => core::ptr::read(HSI_TRIM_ADDR[1] as *const u32),
                    HsiClk::Hz16M => core::ptr::read(HSI_TRIM_ADDR[2] as *const u32),
                    HsiClk::Hz22_12M => core::ptr::read(HSI_TRIM_ADDR[3] as *const u32),
                    HsiClk::Hz24M => core::ptr::read(HSI_TRIM_ADDR[4] as  *const u32),
                }
            }
        }

        pub(super) fn config(&self) {
            let rb = super::Rcc::block();
            defmt::info!("{:x}", rb.icscr.read().bits());
            rb.icscr.write(|w| unsafe {
                // 选择 hsi 频率
                w.hsi_fs().bits(*self as u8);
                // 选择校验值
                w.hsi_trim().bits(self.trim() as u16)
            });
            // 开启hsi
            rb.cr.write(|w| w.hsion().set_bit());
    
            // to check HSIRDY bit weather prepare
            _ = super::common::wait_fun(10000, || {
                rb.cr.read().hsirdy() == true
            });
            defmt::info!("{:x}", rb.icscr.read().bits());
        }
    }
    
    pub struct CLK_LSI;


    pub mod hsi {
        use core::marker::PhantomData;
        use core::marker::ConstParamTy;
        use super::super::Rcc;
        use crate::common::PeriphPtr;

        #[derive(Debug, Clone, Copy, PartialEq, Eq,core::marker::ConstParamTy)]
        pub enum HsiHz {
            Hz4M = 0b000,
            Hz8M = 0b001,
            Hz16M = 0b010,
            Hz22_12M = 0b011,
            Hz24M = 0b100,
        }

        impl<const HZ: HsiHz> crate::rcc::Config for HsiClk<HZ>{
            fn config(en: bool) -> Result<(),()> {
                let reg = Rcc::block();

                if en == false {
                    reg.cr.write(|w| w.hsion().clear_bit());
                    Ok(())
                }
                else {
                    const HSI_TRIM_ADDR: [u32; 5] = [0x1FFF_0F00, 0x1FFF_0F04, 0x1FFF_0F08, 0x1FFF_0F0C, 0x1FFF_0F10];
                    let trim = unsafe {
                        match HZ {
                            HsiHz::Hz4M => core::ptr::read(HSI_TRIM_ADDR[0] as *const u32),
                            HsiHz::Hz8M => core::ptr::read(HSI_TRIM_ADDR[1] as *const u32),
                            HsiHz::Hz16M => core::ptr::read(HSI_TRIM_ADDR[2] as *const u32),
                            HsiHz::Hz22_12M => core::ptr::read(HSI_TRIM_ADDR[3] as *const u32),
                            HsiHz::Hz24M => core::ptr::read(HSI_TRIM_ADDR[4] as  *const u32),
                        }
                    };

                    // 写入时钟校验值
                    reg.icscr.write(|w| unsafe {
                        // 选择 hsi 频率
                        w.hsi_fs().bits(HZ as u8);
                        // 选择校验值
                        w.hsi_trim().bits(trim as u16)
                    });
                    Ok(())
                }
            }
        }

        pub struct HsiClk<const HZ: HsiHz = {HsiHz::Hz8M}>;
        
        #[derive(ConstParamTy, PartialEq, Eq)]
        pub enum HsiDiv {
            HsiDiv1 = 0b000,
            HsiDiv2 = 0b001,
            HsiDiv4 = 0b010,
            HsiDiv8 = 0b011,
            HsiDiv16 = 0b100,
            HsiDiv32 = 0b101,
            HsiDiv64 = 0b110,
            HsiDiv128 = 0b111,
        }
        pub struct HsiSys<HZ: crate::rcc::Config, const DIV: HsiDiv = {HsiDiv::HsiDiv1}>{
            _hz: PhantomData<HZ>,
        }

        impl<HZ: crate::rcc::Config, const DIV: HsiDiv> HsiSys<HZ, DIV> {
            fn config(en: bool) -> Result<(), ()> {
                if en == true {
                    let rb = Rcc::block();
                    unsafe {
                        rb.cr.write(|w| w.hsidiv().bits(DIV as u8));
                    }
                }
                HZ::config(en)
            }
        }
    }
    
    
    pub mod hse{
        use super::super::Rcc;
        use crate::common;

        use crate::common::PeriphPtr;

        pub enum HseBypass{
            HseBypassOscillator = 0,
            HseBypassPulse = 1,
        }
        pub struct HseClk<const HZ: u32 = 8000000>;

        impl<const HZ: u32> super::CLK_HZ for HseClk<HZ> {
            fn hz(&self) -> u32 {
                HZ
            }
    
            fn clk() -> u32 {
                todo!()
            }
        }
    
        impl<const HZ: u32> crate::rcc::Config for HseClk<HZ> {
            fn config(en: bool) -> Result<(), ()> {
                let reg = Rcc::block();
                reg.cr.write(|w| w.hseon().bit(en));
                // Ok(())
                // 软件可置位和清零。进入 stop 模式，硬件清零该位。
                // 如果 HSE 被直接或者间接用作系统时钟，则该位不能被复位。
                if en == false {
                    // 如果 HSE 被直接或者间接用作系统时钟，则该位不能被复位。
                    reg.cr.write(|w| w.hseon().clear_bit());
                    return Ok(());
                }
                else {
                    reg.cr.write(|w| w.hseon().set_bit());
                    // HSE 时钟准备标志位
                    // 硬件置位，表明 HSE 稳定了。
                    if true == common::wait_fun(1000, || reg.cr.read().hserdy() == true){
                        Ok(())
                    }
                    else {
                        Err(())
                    }
                }
            }
        }
    }
   
}

pub(crate) struct Rcc;

impl common::PeriphPtr for Rcc {
    type Target = &'static pac::rcc::RegisterBlock;
    fn block() -> Self::Target {
        unsafe {
            pac::RCC::PTR.as_ref().unwrap()
        }
    }
}

// PLL 可以用来对 HSI 或者 HSE 进行倍频。在使能 PLL 之前，必须对 PLL 进行配置。
// 一旦 PLL 被使能，这 些被配置的寄存器不能被改变。

// pub enum Pll{
//     hsi(clk_select::HsiClk),
//     hse(clk_select::CLK_HSE),
// }

// impl Pll {
//     fn enable(en: bool) {
//         let block = Rcc::block();
//         block.cr.write(|w| w.pllon().bit(en));
//         #[cfg(debug_assertions)]
//         if en == false {
//             // 软件可置位、清零。当进入 stop 模式时，硬件会清零该 位。如果 PLL 时钟被用作系统时钟时，该位不能被复位。
//             if block.cfgr.read().sws().bits() == 0b010 {
//                 panic!("pll clk as system clk")
//             }
//         }
//     }

//     fn config() -> bool {
//         false
//     }
// }

#[derive(PartialEq, Eq)]
pub struct HsiSys{
    clk: clk_select::HsiClk,
    div: clk_select::HsiDiv,
}

impl HsiSys {
    pub fn set_clk(&self, clk: clk_select::HsiClk) -> Self {
        Self { clk: clk, div: self.div }
    }

    pub fn set_div(&self, div: clk_select::HsiDiv) -> Self {
        Self { clk: self.clk, div: div }
    }
}

impl Default for HsiSys {
    fn default() -> Self {
        Self { clk: Default::default(), div: Default::default() }
    }
}

#[derive(PartialEq, Eq)]
pub enum SysClkSource{
    HSISYS(HsiSys),
    HSE,
    PLL_CLK,
    LSI,
    LSE,
}

pub struct Clk{
    pub core: SysClkSource,
}

static mut F_CPU: u32 = 24*1000*1000;

impl Default for Clk {
    fn default() -> Self {
        Self { core: SysClkSource::HSISYS(Default::default()) }
    }
}

static mut CLK_IS_TAKE: bool = false;
impl Clk {
    pub fn take() -> Option<Self>{
        if unsafe {
            CLK_IS_TAKE == false
        }{
            unsafe{ CLK_IS_TAKE = true;}
            Some(Self {
                core: SysClkSource::HSISYS(Default::default())
            })
        }
        else {
            None
        }
    }

    pub fn sys_core_clk_config(&self, cfg: SysClkSource) -> bool {
        let block = Rcc::block();

        if block.cfgr.read().sws() == 0b010 && self.core != SysClkSource::PLL_CLK {
            return false
        }

        match cfg {
            // 内部低速时钟
            SysClkSource::LSI => {
                block.cfgr.write(|w| unsafe {
                    w.sw().bits(0x3)
                });
                unsafe {
                    F_CPU = 32768;
                }
            },
            SysClkSource::LSE => {
                block.cfgr.write(|w| unsafe {
                    w.sw().bits(0b100)
                });
            },
            SysClkSource::PLL_CLK => {
                block.cfgr.write(|w| unsafe {
                    w.sw().bits(0b010)
                });
            },
            SysClkSource::HSE => {
                block.cfgr.write(|w| unsafe {
                    w.sw().bits(0b001)
                });
            },
            SysClkSource::HSISYS(hsisys) => {
                // 配置成 hsi 
                hsisys.clk.config();
                hsisys.div.config();
                block.cfgr.write(|w| unsafe {
                    w.sw().bits(000)
                });
                unsafe {
                    // f_cpu = hsi / div
                    F_CPU = hsisys.clk.hz()/(0x01 << (hsisys.div as u32));
                }
            },
        };
        false
    }
}


pub struct SysClk<T: Config>(T);

impl<T: Config> Config for SysClk<T> {
    fn config(en: bool) -> Result<(), ()> {
        T::config(en)
    }
}