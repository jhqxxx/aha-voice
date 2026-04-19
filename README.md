# aha-voice

A Rust-based voice assistant framework that provides modular components for speech processing, including Voice Activity Detection (VAD), Automatic Speech Recognition (ASR), Large Language Model (LLM) integration, and Text-to-Speech (TTS).

## ⚠️ Current Status: Basic Demo Version

This is currently a **minimal demonstration version** showcasing the core capabilities of the aha-voice framework. The examples provided are beginner-friendly demonstrations that illustrate fundamental concepts but lack production-ready features such as error recovery, performance optimization, and robust state management.

## Features

- 🎤 **Audio Input/Output**: Real-time microphone input and speaker output using cpal
- 🔇 **Voice Activity Detection (VAD)**: Detect speech segments using FireRed VAD model
- 🗣️ **Automatic Speech Recognition (ASR)**: Convert speech to text using Qwen3 ASR
- 🧠 **Large Language Model (LLM)**: Process text with Qwen3.5 model for intelligent responses
- 🔊 **Text-to-Speech (TTS)**: Generate natural speech from text using VoxCPM
- ⚡ **Hardware Acceleration**: Optional CUDA and Metal support for improved performance

## Prerequisites

### System Dependencies

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
No additional system dependencies required.

**Windows:**
No additional system dependencies required (ASIO support included).

### Rust Toolchain

Ensure you have Rust installed (Edition 2024 or later):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Installation

Clone the repository:
```bash
git clone https://github.com/jhqxxx/aha-voice.git
cd aha-voice
```

Build the project:
```bash
cargo build
```

## Configuration

### Hardware Acceleration

Enable GPU acceleration features:

**CUDA (NVIDIA GPUs):**
```bash
cargo build --features cuda
```

**Metal (Apple Silicon):**
```bash
cargo build --features metal
```

## Model Requirements

To run the full examples, you'll need to download the following models:

- **FireRed VAD Model**: Voice activity detection
    - 📥 **Recommended Weights** (Compatible with aha-voice): [jiangjiangaha/FireRedVAD-Stream-VAD](https://www.modelscope.cn/models/jiangjiangaha/FireRedVAD-Stream-VAD/)
    - ℹ️ *Note: The original model weights ([xukaituo/FireRedVAD](https://modelscope.cn/models/xukaituo/FireRedVAD/tree/master/Stream-VAD)) use `.pth.tar` + `.ark`, are not directly compatible with aha-voice. Please use the recommended link above.*
- **Qwen3 ASR Model**: Speech recognition
    - 📥 **Model Weights**: [Qwen/Qwen3-ASR Collection](https://modelscope.cn/collections/Qwen/Qwen3-ASR)
- **Qwen3.5 LLM Model**: Language understanding and generation
    - 📥 **Model Weights**: [Qwen/Qwen3.5 Collection](https://modelscope.cn/collections/Qwen/Qwen35)
- **VoxCPM Model**: Text-to-speech synthesis
    - 📥 **Model Weights**: [VoxCPM Collection](https://modelscope.cn/collections/VoxCPM-359d157eea3849)

Model paths should be provided via command-line arguments as shown in the Quick Start section.

## Quick Start

The project includes progressive examples demonstrating different levels of functionality:

### 1. Audio Input Demo
Capture audio from microphone:
```bash
cargo run --example cpal_in_beginner
```

### 2. Audio Output Demo
Play WAV file through speakers:
```bash
cargo run --example cpal_out_beginner
```

### 3. VAD Demo
Detect speech activity:
```bash
cargo run --features cuda --example vad_beginner --vad-path /path/to/vad/model
```

### 4. ASR Demo
Convert speech to text (VAD + ASR):
```bash
cargo run --features cuda --example asr_beginner --vad-path /path/to/vad/model --asr-path /path/to/qwen3asr/model
```

### 5. LLM Demo
Full voice interaction pipeline (VAD + ASR + LLM):
```bash
cargo run --features cuda --example llm_beginner \
  --vad-path /path/to/vad/model \
  --asr-path /path/to/qwen3asr/model \
  --qwen3.5-path /path/to/qwen3.5/model
```

### 6. TTS Demo
Complete voice assistant with speech output (VAD + ASR + LLM + TTS):
```bash
cargo run --features cuda --example tts_beginner \
  --vad-path /path/to/vad/model \
  --asr-path /path/to/qwen3asr/model \
  --qwen3.5-path /path/to/qwen3.5/model \
  --voxcpm-path /path/to/voxcpm/model
```

## Architecture

The project follows a modular pipeline architecture:

```
Microphone Input → VAD → ASR → LLM → TTS → Speaker Output
```

Each component runs in separate threads for concurrent processing:
- **Audio Capture Thread**: Continuously records microphone input
- **VAD Processing Thread**: Detects speech segments
- **ASR Thread**: Converts detected speech to text
- **LLM Thread**: Generates intelligent responses
- **TTS Thread**: Converts text responses to speech
- **Audio Output Thread**: Plays synthesized speech

## Project Structure

```
aha-voice/
├── examples/
│   ├── cpal_in_beginner.rs    # Basic audio input
│   ├── cpal_out_beginner.rs   # Basic audio output
│   ├── vad_beginner.rs        # Voice activity detection
│   ├── asr_beginner.rs        # Speech recognition
│   ├── llm_beginner.rs        # Language model integration
│   └── tts_beginner.rs        # Text-to-speech synthesis
├── src/
│   ├── lib.rs                 # Core library functions
│   └── main.rs                # Main entry point
├── Cargo.toml                 # Project configuration
└── README.md                  # This file
```

## Future Improvements

This demo version has several areas for enhancement:

### 🎯 Priority Improvements

1. **Error Handling & Recovery**
   - Implement graceful error recovery for model failures
   - Add retry mechanisms for network/model operations
   - Better error messages and logging

2. **Performance Optimization**
   - Optimize audio buffer management to reduce latency
   - Implement efficient memory pooling for audio data
   - Add configurable buffer sizes based on use case

3. **State Management**
   - Replace `Arc<Mutex<>>` with more efficient concurrent data structures
   - Implement proper channel-based communication between threads
   - Add state machine for conversation flow management

4. **Configuration System**
   - Add configuration file support (YAML/TOML)
   - Make sample rates, buffer sizes, and thresholds configurable
   - Support multiple audio devices selection

5. **Audio Processing Enhancements**
   - Add noise reduction and audio enhancement
   - Implement echo cancellation for better UX

### 🚀 Advanced Features

6. **Conversation Context**
   - Maintain conversation history for context-aware responses
   - Implement session management
   - Add user profile support

7. **Streaming Support**
   - Implement streaming ASR for real-time transcription
   - Add streaming LLM and TTS for faster response times
   - Support interruptible speech generation

8. **Multi-language Support**
   - Add language detection
   - Support multilingual ASR and TTS
   - Implement translation capabilities

9. **Testing & Quality**
   - Add unit tests for core functions
   - Implement integration tests
   - Add benchmarking suite

10. **Documentation & Examples**
    - Add API documentation
    - Create advanced usage examples
    - Provide deployment guides

### 🛠️ Engineering Excellence

11. **Monitoring & Observability**
    - Add metrics collection (latency, throughput)
    - Implement structured logging
    - Add health check endpoints

12. **Security**
    - Add input validation and sanitization
    - Implement rate limiting
    - Secure model loading and execution

13. **Deployment**
    - Create Docker containers
    - Add CI/CD pipelines
    - Support cross-compilation

14. **User Experience**
    - Add visual feedback (UI)
    - Implement wake word detection
    - Add customizable voice profiles

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details.

## Acknowledgments

- [candle](https://github.com/huggingface/candle) - A minimalist ML framework for Rust with a focus on performance (including GPU support) and ease of use
- [aha](https://github.com/jhqxxx/aha.git) - AI model inference library built on top of Candle.
- [cpal](https://github.com/RustAudio/cpal) - Cross-platform audio library
- [hound](https://github.com/ruuda/hound) - WAV audio file reader/writer
- [clap](https://github.com/clap-rs/clap) - Command line argument parser
