# rhess

> [!NOTE]
> 
> 本项目为 **2025 秋季学期 嵌入式系统与设计** 的课程作业。以下为成员学号： `0055713db0a7edac6c4d1743a6ca0fcd9aebe8b6d2656fb03cb4dfadc731ed5f`, `200f1716837e93e0f5aa07b5ec8a43d31646144c85fdbb512f5b3f718f5e2751` (SHA-256).

一款基于 STM32F407ZGT6 的裸机国际象棋游戏。

## 构建

- 安装 [Rust](https://rust-lang.org/learn/get-started/)
- 安装目标架构：`rustup target add thumbv7em-none-eabihf`
- 调试构建（迭代快）：`cargo build --target thumbv7em-none-eabihf`
- 发布构建（体积小/性能高）：`cargo build --release --target thumbv7em-none-eabihf`

工程为 `no_std`，烧录与探针配置请在本地 `.cargo/config.toml` 中设置。`memory.x` 已匹配 STM32F407ZGT6 的闪存与 SRAM 布局。
