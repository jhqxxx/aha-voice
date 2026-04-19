# aha-voice

一个基于 Rust 的语音助手框架，提供模块化的语音处理组件，包括语音活动检测（VAD）、自动语音识别（ASR）、大语言模型（LLM）集成和文本转语音（TTS）。

## ⚠️ 当前状态：基础演示版本

这是一个**最小化的演示版本**，展示了 aha-voice 框架的核心功能。提供的示例是面向初学者的演示，说明了基本概念，但缺乏生产级功能，如错误恢复、性能优化和健壮的状态管理。

## 功能特性

- 🎤 **音频输入/输出**：使用 cpal 实现实时麦克风输入和扬声器输出
- 🔇 **语音活动检测（VAD）**：使用 FireRed VAD 模型检测语音片段
- 🗣️ **自动语音识别（ASR）**：使用 Qwen3 ASR 将语音转换为文本
- 🧠 **大语言模型（LLM）**：使用 Qwen3.5 模型处理文本并生成智能回复
- 🔊 **文本转语音（TTS）**：使用 VoxCPM 从文本生成自然语音
- ⚡ **硬件加速**：可选的 CUDA 和 Metal 支持以提升性能

## 前置要求

### 系统依赖

**Linux (Ubuntu/Debian):**
```bash
sudo apt install libasound2-dev
```

**Linux (Fedora/CentOS/RHEL):**
```bash
sudo yum install alsa-lib-devel
# or
sudo dnf install alsa-lib-devel
```

**macOS:**
无需额外的系统依赖。

**Windows:**
无需额外的系统依赖（已包含 ASIO 支持）。

### Rust 工具链

确保已安装 Rust（Edition 2024 或更高版本）：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 安装

克隆仓库：
```bash
git clone https://github.com/jhqxxx/aha-voice.git
cd aha-voice
```

构建项目：
```bash
cargo build
```

## 配置

### 硬件加速

启用 GPU 加速功能：

**CUDA (NVIDIA GPU):**
```bash
cargo build --features cuda
```

**Metal (Apple Silicon):**
```bash
cargo build --features metal
```

## 模型要求

要运行完整示例，您需要下载以下模型：

- **FireRed VAD 模型**：语音活动检测
   - 📥 **推荐权重** (aha-voice 可解析): [jiangjiangaha/FireRedVAD-Stream-VAD](https://www.modelscope.cn/models/jiangjiangaha/FireRedVAD-Stream-VAD/)
   - ℹ️ *注意: 原始模型权重 ([xukaituo/FireRedVAD](https://modelscope.cn/models/xukaituo/FireRedVAD/tree/master/Stream-VAD)) 使用 `.pth.tar` + `.ark`，无法直接被 aha-voice 解析，请使用上述推荐链接。*
- **Qwen3 ASR 模型**：语音识别
   - 📥 **模型权重**: [Qwen/Qwen3-ASR Collection](https://modelscope.cn/collections/Qwen/Qwen3-ASR)
- **Qwen3.5 LLM 模型**：语言理解和生成
   - 📥 **模型权重**: [Qwen/Qwen3.5 Collection](https://modelscope.cn/collections/Qwen/Qwen35)
- **VoxCPM 模型**：文本转语音合成
   - 📥 **模型权重**: [VoxCPM Collection](https://modelscope.cn/collections/VoxCPM-359d157eea3849)

模型路径应通过命令行参数提供，如快速开始部分所示。

## 快速开始

本项目包含渐进式的示例，展示不同级别的功能：

### 1. 音频输入演示
从麦克风捕获音频：
```bash
cargo run --example cpal_in_beginner
```

### 2. 音频输出演示
通过扬声器播放 WAV 文件：
```bash
cargo run --example cpal_out_beginner
```

### 3. VAD 演示
检测语音活动：
```bash
cargo run --features cuda --example vad_beginner --vad-path /path/to/vad/model
```

### 4. ASR 演示
将语音转换为文本（VAD + ASR）：
```bash
cargo run --features cuda --example asr_beginner --vad-path /path/to/vad/model --asr-path /path/to/qwen3asr/model
```

### 5. LLM 演示
完整的语音交互管道（VAD + ASR + LLM）：
```bash
cargo run --features cuda --example llm_beginner \
  --vad-path /path/to/vad/model \
  --asr-path /path/to/qwen3asr/model \
  --qwen3.5-path /path/to/qwen3.5/model
```

### 6. TTS 演示
带语音输出的完整语音助手（VAD + ASR + LLM + TTS）：
```bash
cargo run --features cuda --example tts_beginner \
  --vad-path /path/to/vad/model \
  --asr-path /path/to/qwen3asr/model \
  --qwen3.5-path /path/to/qwen3.5/model \
  --voxcpm-path /path/to/voxcpm/model
```

## 架构设计

项目采用模块化管道架构：

```
麦克风输入 → VAD → ASR → LLM → TTS → 扬声器输出
```

每个组件在独立的线程中运行以实现并发处理：
- **音频捕获线程**：持续录制麦克风输入
- **VAD 处理线程**：检测语音片段
- **ASR 线程**：将检测到的语音转换为文本
- **LLM 线程**：生成智能回复
- **TTS 线程**：将文本回答转换为语音
- **音频输出线程**：播放合成的语音

## 项目结构

```
aha-voice/
├── examples/
│   ├── cpal_in_beginner.rs    # 基础音频输入
│   ├── cpal_out_beginner.rs   # 基础音频输出
│   ├── vad_beginner.rs        # 语音活动检测
│   ├── asr_beginner.rs        # 语音识别
│   ├── llm_beginner.rs        # 语言模型集成
│   └── tts_beginner.rs        # 文本转语音合成
├── src/
│   ├── lib.rs                 # 核心库函数
│   └── main.rs                # 主入口点
├── Cargo.toml                 # 项目配置
└── README.md                  # 本文件
```

## 后续改进方向

这个演示版本在多个方面有提升空间：

### 🎯 优先改进项

1. **错误处理与恢复**
   - 为模型失败实现优雅的错误恢复机制
   - 为网络/模型操作添加重试机制
   - 提供更好的错误消息和日志记录

2. **性能优化**
   - 优化音频缓冲区管理以减少延迟
   - 为音频数据实现高效的内存池
   - 根据用例添加可配置的缓冲区大小

3. **状态管理**
   - 用更高效的并发数据结构替换 `Arc<Mutex<>>`
   - 实现基于通道的线程间通信
   - 为对话流程管理添加状态机

4. **配置系统**
   - 添加配置文件支持（YAML/TOML）
   - 使采样率、缓冲区大小和阈值可配置
   - 支持多音频设备选择

5. **音频处理增强**
   - 添加降噪和音频增强功能
   - 实现回声消除以改善用户体验

### 🚀 高级功能

6. **对话上下文**
   - 维护对话历史以实现上下文感知回复
   - 实现会话管理
   - 添加用户档案支持

7. **流式支持**
   - 实现流式 ASR 以实现实时转录
   - 添加流式 LLM及TTS 以加快响应时间
   - 支持可中断的语音生成

8. **多语言支持**
   - 添加语言检测
   - 支持多语言 ASR 和 TTS
   - 实现翻译功能

9. **测试与质量**
   - 为核心函数添加单元测试
   - 实现集成测试
   - 添加基准测试套件

10. **文档与示例**
    - 添加 API 文档
    - 创建高级用法示例
    - 提供部署指南

### 🛠️ 工程卓越

11. **监控与可观测性**
    - 添加指标收集（延迟、吞吐量）
    - 实现结构化日志
    - 添加健康检查端点

12. **安全性**
    - 添加输入验证和清理
    - 实现速率限制
    - 安全的模型加载和执行

13. **部署**
    - 添加 CI/CD 流水线
    - 支持交叉编译

14. **用户体验**
    - 添加视觉反馈（UI）
    - 实现唤醒词检测
    - 添加可定制的语音配置文件

## 贡献

欢迎贡献！请随时提交 Pull Request 或为 Bug 和功能请求开启 Issue。

## 许可证

本项目采用 Apache License 2.0 许可证 - 详见 LICENSE 文件。

## 致谢

- [candle](https://github.com/huggingface/candle) - 一个专注于性能和易用性的 Rust 极简机器学习框架。
- [aha](https://github.com/jhqxxx/aha.git) - 基于 Candle 构建的 AI 模型推理库
- [cpal](https://github.com/RustAudio/cpal) - 跨平台音频库
- [hound](https://github.com/ruuda/hound) - WAV 音频文件读写库
- [clap](https://github.com/clap-rs/clap) - 命令行参数解析器
