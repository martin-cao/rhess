# rhess

> [!NOTE]
> 
> 本项目为 **2025 秋季学期 嵌入式系统与设计** 的课程作业。以下为成员学号： `0055713db0a7edac6c4d1743a6ca0fcd9aebe8b6d2656fb03cb4dfadc731ed5f`, `200f1716837e93e0f5aa07b5ec8a43d31646144c85fdbb512f5b3f718f5e2751` (SHA-256).

一款基于 STM32F407ZGT6 的裸机国际象棋游戏。

## 特性

- 完整国际象棋规则：合法着法生成、升变、将军/将死判断
- 板载 LCD 界面：显示当前行棋方、子力差、上一步高亮
- 轻量 AI 搜索，可作为对手或双 AI 对弈
- 四种模式：人机/人人/机人/机机（AI 回合自带最小间隔，便于观看）

## 操作说明

- 对局导航：KEY1 左，KEY2 下，KEY3 上，KEY4 右
- KEY1 长按：选中/取消选中棋子；KEY2 长按：提交走子
- 升变：短按 KEY1..KEY4 依次选择 车/马/象/后
- 启动菜单：KEY3 上移，KEY2 下移，KEY1 确认

## 硬件信息

- 目标 MCU：STM32F407ZGT6（外部 25 MHz HSE，168 MHz SYSCLK）
- 显示：FSMC + SSD1963 驱动的 480x272 LCD
- 存储布局：见 `memory.x`（闪存/SRAM）

## 构建

- 安装 [Rust](https://rust-lang.org/learn/get-started/)
- 安装目标架构：`rustup target add thumbv7em-none-eabihf`
- 调试构建（迭代快）：`cargo build --target thumbv7em-none-eabihf`
- 发布构建（体积小/性能高）：`cargo build --release --target thumbv7em-none-eabihf`

工程为 `no_std`，烧录与探针配置请在本地 `.cargo/config.toml` 中设置。`memory.x` 已匹配 STM32F407ZGT6 的闪存与 SRAM 布局。
