
## 简介
该库提供 py32f030 用

## 安装环境
### [安装 Rust](https://www.rust-lang.org/tools/install)
#### Mac/Linux
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
#### [Windows](https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe)

### 设置 Rust night 版本
```bash
rustup default nightly
```
### 安装 Cortex-M0 编译工具
```bash
rustup target add thumbv6m-none-eabi
```
### 安装 [Probe-rs](https://probe.rs/docs/getting-started/installation/#homebrew)
Probe-rs 是一个优秀的固件下载和日志调试工具
#### Mac/Linux
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh \
    | sh
```
#### Windows
```bash
irm https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.ps1 | iex
```

## 其他工具（暂时可不用安装）
### cargo tools
```bash
cargo install cargo-get
brew install llvm
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
- [ ] adc
- [ ] flash
- [ ] spi


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
$ cargo r --example                    
error: "--example" takes one argument.
Available examples:
    blinky
    blinky_lite
    clock
    embassy_delay
    embassy_dma_mem2mem
    embassy_exit
    embassy_uart
    key
    rcc
    uart
``` 


##  关于
公众号：`Rust嵌入式`
<img src="https://s.imgkb.xyz/i/abcdocker/2024/07/20/669bac54b9156.jpg" alt="Rust嵌入式" style="display: block; margin: 0 auto;">
