
## 简介
该库提供 py32f030 芯片使用，目前适配了一些基本的外设驱动

## 安装环境
### [安装 Rust](https://www.rust-lang.org/tools/install)
#### Mac/Linux
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
#### [Windows](https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe)

### 设置 Rust night 版本
Rust 的嵌入式开发环境需要是 nightly 版本
```bash
rustup default nightly
```
### 安装 Cortex-M0 编译工具
```bash
rustup target add thumbv6m-none-eabi
```

### 测试编译环境
执行下面的命令，没有报错说明rust编译环境安装正常
```
cd py32f030-hal
cargo build
```

### 查看本机Rust 版本
```bash
➜  py32f030-hal git:(main) ✗ rustup --version
rustup 1.27.1 (54dd3d00f 2024-04-24)
info: This is the version for the rustup toolchain manager, not the rustc compiler.
info: The currently active `rustc` version is `rustc 1.82.0-nightly (6de928dce 2024-08-18)`
```

### 安装 [Probe-rs](https://probe.rs/docs/getting-started/installation/#homebrew)
Probe-rs 是一个优秀的固件下载和日志调试工具, 详细安装和功能请点击[页面](https://probe.rs/docs/getting-started/installation/#using-install-scripts)查看。
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

## 其他工具（暂时可不用安装）
### cargo tools
```bash
cargo install cargo-get
brew install llvm
```

### 串口烧录工具
1. python工具 puyaisp 安装
```
pip install puyaisp
pip install pyusb pyserial hid
```
烧录：
```bash
# 同时按下 boot 和 RST 按键，然后先释放 RST，然后释放 Boot 即可，然后执行以下命令
puyaisp -f blink.bin
```
2. rust 工具 pyisp
使用 pyisp rust 串口工具烧录 bin 文件 [pyisp](https://github.com/hysonglet/pyisp.git)
``` bash
# 单次下载
pyisp -s tty.usbserial-130 -g -f test.bin
# 多次下载
pyisp -s COM4 -g -c -f test.bin
```


### Jlink 连接运行并查看日志
1. 使用cargo run命令
```
cargo r  --example embassy_uart
```
2. 使用 probe-rs 运行
```
probe-rs run --chip PY32F030x8 target/thumbv6m-none-eabi/debug/examples/embassy_uart
```

## 外设驱动

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

## TODO
- LPTimer
- Flash
- Clock -> 48M
- spi test

## examples

### 执行
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

## 测试开发板
<img src="https://s.imgkb.xyz/i/abcdocker/2025/01/01/67752f84dc98a.png" alt="Py32_Rust_Dev 1.2" title="Py32_Rust_Dev 1.2" />


##  关于
公众号：`Rust嵌入式`
<img src="https://s.imgkb.xyz/i/abcdocker/2024/07/20/669bac54b9156.jpg" alt="Rust嵌入式" style="display: block; margin: 0 auto;">
