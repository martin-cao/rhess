# rhess

[[简体中文](./README_zh-CN.md)]

> [!NOTE]
> 
> This is an course-design project for **2025 Fall** _Embedded System and Design_ by student ID: `0055713db0a7edac6c4d1743a6ca0fcd9aebe8b6d2656fb03cb4dfadc731ed5f`, `200f1716837e93e0f5aa07b5ec8a43d31646144c85fdbb512f5b3f718f5e2751` (SHA-256).

rhess is a bare-metal chess game for an STM32F407ZGT6 board with a 480x272 LCD.

## Features

- Full chess rules with legal move generation, promotion, check detection
- On-board LCD UI with side info (turn, material diff, last move highlight)
- AI opponent powered by a lightweight search (configurable depth)
- Four start modes: Human vs Human, Human vs Computer, Computer vs Human, Computer vs Computer (AI side delay to make moves visible)

## Controls

- In-game navigation: KEY1 left, KEY2 down, KEY3 up, KEY4 right
- KEY1 long press: select/deselect piece; KEY2 long press: submit move
- Promotion: press KEY1..KEY4 for Rook/Knight/Bishop/Queen in order
- Start menu: KEY3 up, KEY2 down, KEY1 confirm

## Hardware

- Target MCU: STM32F407ZGT6 (external 25 MHz HSE, 168 MHz SYSCLK)
- Display: 480x272 LCD via FSMC + SSD1963 controller
- Memory layout: see `memory.x` for flash/SRAM regions

## Build

- Install [Rust](https://rust-lang.org/learn/get-started/)
- Install the ARM target once: `rustup target add thumbv7em-none-eabihf`
- Debug build (faster iteration): `cargo build --target thumbv7em-none-eabihf`
- Release build (smaller/faster image): `cargo build --release --target thumbv7em-none-eabihf`

The project is `no_std`; flashing and probe configuration are expected to be handled via your preferred runner in `.cargo/config.toml`. The provided `memory.x` matches the STM32F407ZGT6 layout.
