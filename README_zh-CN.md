# rhess

[[English](./README.md)]

> [!NOTE]
> 
> 本项目为 **2025 秋季学期 嵌入式系统与设计** 的课程作业。以下为成员学号： `0055713db0a7edac6c4d1743a6ca0fcd9aebe8b6d2656fb03cb4dfadc731ed5f`, `200f1716837e93e0f5aa07b5ec8a43d31646144c85fdbb512f5b3f718f5e2751` (SHA-256)。

基于 STM32F407ZGT6 和 480x272 LCD（FSMC + SSD1963）的裸机国际象棋，使用 Rust `no_std` 编写。

## 亮点

- 完整规则：合法着法生成、升变、将军/将死处理
- 四种模式（人人/人机/机人/机机），AI 搜索深度和动作间隔可调
- LCD 界面：当前行棋方、子力差、上一步高亮、升变选择
- 四键输入：方向移动 + 长按提交；启动菜单也支持同一套按键
- 基于 `stm32f4xx-hal` 的板级支持包，串口/RTT 日志，`memory.x` 对齐 STM32F407ZGT6

## 硬件信息

- MCU：STM32F407ZGT6（25 MHz HSE，168 MHz SYSCLK）
- 显示：FSMC + SSD1963 驱动的 480x272 LCD
- 按键：PE2/PE3/PE4/PA0（上拉、低电平有效，对应 KEY1..KEY4）
- LED：PC0、PF10、PB0、PB1（低电平点亮）

## 操作

- 对局导航：KEY1 左，KEY2 下，KEY3 上，KEY4 右
- KEY1 长按：选中/取消棋子；KEY2 长按：提交走子
- 升变：短按 KEY1..KEY4 依次选择 车/马/象/后
- 启动菜单：KEY3 上移，KEY2 下移，KEY1 确认

## 项目结构

- `src/main.rs`：入口，初始化板卡、模式选择与循环
- `src/board.rs`：时钟、GPIO、FSMC LCD、USART1、按键、LED 等板级初始化
- `src/chess_core/`：棋盘表示、规则与着法生成
- `src/game.rs`：回合状态机与 AI 集成
- `src/ui/`：棋盘及侧边信息的绘制工具
- `src/drivers/`：LCD、按键、LED、串口、延时等驱动
- `src/start_menu*.rs`：启动菜单渲染与选择逻辑

## 构建与烧录

1. 安装目标架构：`rustup target add thumbv7em-none-eabihf`
2. 调试构建（迭代快）：`cargo build --target thumbv7em-none-eabihf`
3. 发布构建：`cargo build --release --target thumbv7em-none-eabihf`
4. 使用 probe-rs 烧录示例：`cargo flash --release --chip STM32F407ZG`
5. 如在 `.cargo/config.toml` 配置了 runner（如 `probe-rs run` / `probe-run`），可用 `cargo run --target thumbv7em-none-eabihf` 一键构建并下载

## 调试

- RTT 日志：用 `probe-rs attach --chip STM32F407ZG --rtt` 查看
- VS Code：`.vscode/launch.json` 提供 `probe-rs-debug` 模板，按需修改 `chip`、`programBinary` 与 `speed`

## 许可证

GPL-3.0-only，详见 `LICENSE.txt`。
