# rhess

[[简体中文](./README_zh-CN.md)]

rhess is a bare-metal chess game for an STM32F407ZGT6 board with a 480x272 LCD.

## Build

- Install [Rust](https://rust-lang.org/learn/get-started/)
- Install the ARM target once: `rustup target add thumbv7em-none-eabihf`
- Debug build (faster iteration): `cargo build --target thumbv7em-none-eabihf`
- Release build (smaller/faster image): `cargo build --release --target thumbv7em-none-eabihf`

The project is `no_std`; flashing and probe configuration are expected to be handled via your preferred runner in `.cargo/config.toml`. The provided `memory.x` matches the STM32F407ZGT6 layout.
