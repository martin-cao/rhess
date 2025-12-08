# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Embedded Rust entry (`main.rs`) and future drivers; `memory.x` provides the STM32F407ZGT6 linker script.
- `Cargo.toml`: Crate metadata and MCU/HAL dependencies; adjust features here when new peripherals are used.

## Build, Test, and Development Commands
- `cargo build --release --target thumbv7em-none-eabihf`: Produce optimized firmware binary for F407 with hardware FPU.
- `cargo build --target thumbv7em-none-eabihf`: Faster debug build during driver bring-up.
- `cargo fmt`: Enforce Rust formatting before committing.
- `cargo clippy --target thumbv7em-none-eabihf -Znextest`: Lint for embedded-safe patterns; fix warnings or justify.
- `cargo size --bin dino -- -A`: Inspect code size after release builds (requires `cargo-binutils`).

## Coding Style & Naming Conventions
- Follow `cargo fmt` defaults; prefer small, single-purpose modules per peripheral (e.g., `gpio.rs`, `lcd.rs`).
- Use `CamelCase` for types, `snake_case` for modules/functions, `SCREAMING_SNAKE_CASE` for consts/register masks.
- Prefer `embedded-hal` traits in public APIs; isolate MCU-specific code behind HAL modules.
- Keep unsafe blocks minimal and commented with rationale linked to schematic/errata lines.

## Testing Guidelines
- Add unit tests for pure logic where possible (`cargo test` on host); hardware-facing code should expose small, mockable abstractions.
- For on-target checks, add lightweight self-tests (e.g., GPIO loopback, LCD ID read) behind feature flags to keep release image lean.

## Commit & Pull Request Guidelines
- Commits: imperative, scoped messages (e.g., `Add FSMC LCD init sequence`); group related peripheral changes together.
- PRs: include summary, affected peripherals, and how verified (sim/host test/bench run/board smoke test). Link schematic section or lab reference for timing-critical code and attach logs or photos when hardware output matters.

## Security & Configuration Tips
- Keep `memory.x` aligned with actual flash/SRAM layout; verify stack/heap fit before enabling new RT tasks.
- Do not commit board-specific secrets (probe serials, Wi-Fi credentials). Use `.cargo/config.toml` locally for probe/runner settings.
