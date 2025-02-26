
## 💁 Overview
This repository provides a driver layer for the Py32f030 chip. It currently supports most peripherals and provides many friendly interfaces. By calling the upper-level interfaces, the peripherals of the microcontroller can be easily put into operation.

## 💻 Development environment
Installing the Rust embedded compilation environment for Py32F030 is very simple. You need to install the Rust compilation chain and some tools
### [Rust](https://www.rust-lang.org/tools/install)
#### Mac/Linux
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
#### [Windows](https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe)
The installation of Rust on Windows is a little bit complicated. You can read [this website](https://forge.rust-lang.org/infra/other-installation-methods.html) to learn how to install it.

### Rust nightly version
After installing rust, you need to switch to the night version to compile some embedded rust code
```bash
rustup default nightly
```
### Installing the Cortex-M0 Compiler Tools
```bash
rustup target add thumbv6m-none-eabi
```

### Check the environment
Execute the following command. If there is no error, it means that the rust compilation environment is installed normally.
```
git clone https://github.com/hysonglet/py32f030-hal.git
cd py32f030-hal
cargo build
```

### Check Rust version
```bash
➜  py32f030-hal git:(main) ✗ rustup --version
rustup 1.27.1 (54dd3d00f 2024-04-24)
info: This is the version for the rustup toolchain manager, not the rustc compiler.
info: The currently active `rustc` version is `rustc 1.82.0-nightly (6de928dce 2024-08-18)`
```

### 安装 [Probe-rs](https://probe.rs/docs/getting-started/installation/#homebrew)
Probe-rs is an excellent firmware download and log debugging tool. For detailed of installation and how to use, please click [this link](https://probe.rs/docs/getting-started/installation/#using-install-scripts) to see more。
#### Mac/Linux
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh \
    | sh
```
#### Windows
```bash
cargo install cargo-binstall
cargo binstall probe-rs-tools
```

## Other tools (not required for now)
### cargo tools
```bash
cargo install cargo-get
brew install llvm
```

### Serial port burning tool
There are two tools here, you can choose a tool written in Python or a serial port tool written in Rust.
1. python sccript: puyaisp
```
pip install puyaisp
pip install pyusb pyserial hid
```
burn：
```bash
# Press the boot and RST buttons at the same time, then release RST first, then release Boot, and then execute the following command
puyaisp -f blink.bin
```
2. rust tools: [pyisp](https://github.com/hysonglet/pyisp.git)
Use pyisp rust serial port tool to burn bin file, 
``` bash
# One-time download
pyisp -s tty.usbserial-130 -g -f test.bin
# Repeated download
pyisp -s COM4 -g -c -f test.bin
```


### Jlink connection operation and view log
Of course, it is not limited to jlink, stlink is also a good choice. Sometimes we use the following two methods to download and view the running logs
1. cargo run
```
cargo r  --example embassy_uart
```
2. probe-rs run
```
probe-rs run --chip PY32F030x8 target/thumbv6m-none-eabi/debug/examples/embassy_uart
```

## Supported peripheral drivers

- [x] gpio
- [x] exit
- [x] timer
- [x] i2c
- [x] clock
- [x] embassy
- [x] dma
- [x] usart
- [x] adc
- [x] flash
- [x] spi
- [x] crc
- [x] rtc
- [x] iwatchdog
- [x] Flash

## TODO
- LPTimer
- Clock -> 48M
- spi

## Examples

### Run
```bash
# run
cargo run --example blinky
# build
cargo build --release --example blinky
```

### Example list
```bash
py32f030-hal git:(main) ✗ cargo r --example
error: "--example" takes one argument.
Available examples:
   adc_block
   advanced_timer_block
   advanced_timer_block_2
   bit_test
   blinky
   block_uart
   clock
   crc
   dma_mem2mem
   embassy_adc
   embassy_delay
   embassy_dma_mem2mem
   embassy_exit
   embassy_i2c
   embassy_iwdg
   embassy_pwm
   embassy_rtc
   embassy_ssd1309
   embassy_uart
   hello_world
   i2c_master_block
   key
   rtc_block
   uart
```

## Evaluation Board
<img src="https://s.imgkb.xyz/i/abcdocker/2025/01/01/67752f84dc98a.png" alt="Py32_Rust_Dev 1.2" title="Py32_Rust_Dev 1.2" />

## Wechat blog
公众号：`Rust嵌入式`
<img src="https://s.imgkb.xyz/i/abcdocker/2024/07/20/669bac54b9156.jpg" alt="Rust嵌入式" style="display: block; margin: 0 auto;">

## How to buy
We currently provide Taobao links, you can choose any one to buy
- [Red Board](https://item.taobao.com/item.htm?abbucket=3&id=870372823551&ns=1&pisk=g_4ZEUV7dq2BYJFzdz02YPC73Jgt-qW57rMjiSVm1ADiCScmuJ2f6ANcB-zqKSE16m69gCUUUET1BGFDuqgcFT_5P5EtkqXSqW-3hdhSwfbjsmvnWjiOd69CP5FtHCJcNkbW38cpMIxmn-0nxjcoof0mmWfEGbLmoAmDKHcoKqD0nmYntblJsEDDopfE6bHDSFmDtDcmahcgnqfE-bHnnqmm6bN46UkZjsltBejv0vmZE5DezWUEsrKT_v7cuyP-2YYSLEY08fVISNCBSak3fo4s-PXHh2V8ARcEzZRqTuPnSfyAyF3gZWqmbrSXwxEUtuiUvC1tTPPzuboC8pcr4y4sWkWyPYqLiro36T--OoV75oy1pED_4W2EVy9BP24aQPmErgRXHX4VLrEwnnoi9Xk5T6kW_3HU4z6G5nKxY2GEF_5eDnniDXk5TCxvDDlsTY1__&priceTId=2147847117405862249073402ec550&spm=a21n57.1.hoverItem.2&utparam=%7B%22aplus_abtest%22%3A%22869e3ae3cd2e9d2f08a139771bd78df4%22%7D&xxc=taobaoSearch&skuId=5702998681883)
- [Red Board](https://item.taobao.com/item.htm?abbucket=3&id=873483284901&ns=1&pisk=gQSoE2iF-IGsgeEgVjt7WM_HxuaYe3tBP6np9HdUuIRj2eRLP6Yh9sTF23BRiB5OtpIRvMbjxO6C28tLF36WAHPT6lBhFTtQMED4mMxV3L9zv4oE43t2zxwYDlEOFtDJURB4XgCN0Cvo8D5ezER2CIJyTBoznEJ9LHoy80uqgIO2TH8eUKu2QLmEYDoznxJwdDleaek23LpeUH-FTtyDdIJEOWnk1g7P0S7J9OWpz8svEUANUQWReikJy46WiflGmOGksTuIYDSDEUSC7qQCPH7PesYOr5mkAT71Od5ra7vcqOjh7srxqHXV7MxV0SkvaN6cx3Izpu-Vq_jyo6kQHISHI9YOkJuMMwWfmEjuWv9f29SWWhrsaBQAIMvhAWZcsT5l7FSrZgWSuV5aQD94piuIRUJXnCCPHBlGLZOyb-2mWkLyhdAYn-0QcUJXn6w0nV39zK9-s&priceTId=2147847117405862249073402ec550&skuId=5873994718463&spm=a21n57.1.hoverItem.34&utparam=%7B%22aplus_abtest%22%3A%22ff785536f5ab6bb6f59d61effd2b2e31%22%7D&xxc=taobaoSearch)
- [Stlink](https://item.taobao.com/item.htm?abbucket=3&id=870372823551&ns=1&pisk=gV8teqfMiXFtjPk-QFo3o97RRAlHhDAZIdR7otXgcpppG94m_NmVkKBp3OjG5O4AkIp2nKdq_s6XhKBDjD0k_C7VlYjxr4Aw22ctpdC1G6GfG_zjtGw91NP1lYDokkVCbAbXIvN6j2ZCa911GO6j9w1Vw1_fc1OI9s10Coa6hXdCLsq_lsasR61P1R_blPOCd_10hs611y_CLs_fhK_j9MGCDA63G6UvVwfF2CyxYzzyJ1IOHrXLkg8TrMBWV9aj-j5GXlA1praXRHf84RW-JJ6HGQ-fvE338ZRH2KKXdjafHH1Je_viRPQXApLRVBk87OKBQEQG4qFA93dBBFJjHlpvdQ-P5FM4SO-XOE6H5j4AF3APJ68mLP6vAnTGbZysB6L9wEKA4B8kyHJQETC0fXhL0oS1TR1XiKTZgwId9TcsWorVc_5dEXnY0oS6s6Bo1XE40MPC.&priceTId=2147847117405862249073402ec550&skuId=5716683933293&spm=a21n57.1.hoverItem.2&utparam=%7B%22aplus_abtest%22%3A%22869e3ae3cd2e9d2f08a139771bd78df4%22%7D&xxc=taobaoSearch)