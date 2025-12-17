# rhess

[[简体中文](./README_zh-CN.md)]

> [!NOTE]
> 
> This is an course-design project for **2025 Fall** _Embedded System and Design_ by student ID: `0055713db0a7edac6c4d1743a6ca0fcd9aebe8b6d2656fb03cb4dfadc731ed5f`, `200f1716837e93e0f5aa07b5ec8a43d31646144c85fdbb512f5b3f718f5e2751` (SHA-256).

Bare-metal chess for STM32F407ZGT6 with a 480x272 LCD (FSMC + SSD1963), written in Rust `no_std`.

## Highlights

- Full chess rules: legal move generation, promotion, and check/checkmate handling
- Four play modes (HvH, HvC, CvH, CvC) with configurable AI depth and move delay for readability
- LCD UI with turn indicator, material difference, last-move highlight, and promotion picker
- Four-key input scheme: directional navigation plus long-press submit; start menu supports the same keys
- Board support built on `stm32f4xx-hal`; UART/RTT logging and `memory.x` aligned to STM32F407ZGT6

## Hardware Notes

- MCU: STM32F407ZGT6 (25 MHz HSE, 168 MHz SYSCLK)
- Display: 480x272 panel via FSMC + SSD1963 controller
- Inputs: four keys on PE2/PE3/PE4/PA0 (pull-ups, active-low, mapped as KEY1..KEY4)
- LEDs: PC0, PF10, PB0, PB1 (active-low)

## Controls

- Board navigation: KEY1 left, KEY2 down, KEY3 up, KEY4 right
- KEY1 long press: select/deselect piece; KEY2 long press: submit move
- Promotion: short-press KEY1..KEY4 for Rook/Knight/Bishop/Queen
- Start menu: KEY3 up, KEY2 down, KEY1 confirm

## Project Layout

- `src/main.rs`: entry point; initializes board, start menu, and game loop
- `src/board.rs`: board bring-up (clocks, GPIO, FSMC LCD, USART1, buttons, LEDs)
- `src/chess_core/`: chess rules, board representation, move generation
- `src/game.rs`: turn handling, AI integration, and state transitions
- `src/ui/`: rendering helpers for the board and side info
- `src/drivers/`: LCD, buttons, LEDs, serial, delay drivers
- `src/start_menu*.rs`: start menu rendering and selection logic

## Build & Flash

1. Install the ARM target: `rustup target add thumbv7em-none-eabihf`
2. Debug build for quick iterations: `cargo build --target thumbv7em-none-eabihf`
3. Release build for deployment: `cargo build --release --target thumbv7em-none-eabihf`
4. Flash with probe-rs tools (example): `cargo flash --release --chip STM32F407ZG`
5. With a runner in `.cargo/config.toml` (e.g., `probe-rs run` or `probe-run`), `cargo run --target thumbv7em-none-eabihf` will build and load in one step

## Debugging

- RTT logging via `rtt-target`; view with `probe-rs attach --chip STM32F407ZG --rtt`
- Visual Studio Code: `.vscode/launch.json` ships a `probe-rs-debug` template—set `chip`, `programBinary`, and optional `speed` to match your probe

## License

GPL-3.0-only, see `LICENSE.txt`.
