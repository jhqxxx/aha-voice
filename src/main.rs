use aha::Tensor;
use aha_voice::{
    asr::AsrProcessor,
    audio::{aec::AecProcessor, input::AudioInput, output::AudioOutput},
    llm::LlmProcessor,
    pipeline::message::{AudioRingBuffer, ControlCommand},
    tts::TtsProcessor,
    vad::VadProcessor,
};
use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to VAD model directory
    #[arg(short, long = "vad-path")]
    vad_path: String,

    /// Path to qwen3asr model directory
    #[arg(short = 'a', long = "asr-path")]
    qwen3asr_path: String,

    /// Path to qwen3_5_path model directory
    #[arg(short = 'l', long = "qwen3.5-path")]
    qwen3_5_path: String,

    /// Path to voxcpm model directory
    #[arg(short = 't', long = "voxcpm-path")]
    voxcpm_path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mike_sample_rate = 16000u32;
    let mike_channels = 1u16;
    let ring_buffer_capacity = mike_sample_rate as usize * mike_channels as usize * 5;
    let mike_rb = Arc::new(AudioRingBuffer::new(ring_buffer_capacity));
    let (mute_tx_ctrl, mute_rx_ctrl) = mpsc::channel::<ControlCommand>(20);
    let (aec_tx_ctrl, aec_rx_ctrl) = broadcast::channel::<ControlCommand>(20);
    let aec_rx_ctrl2 = aec_tx_ctrl.subscribe();
    let _input = AudioInput::new(mike_rb.clone(), mike_channels, mike_sample_rate)?;
    let clean_rb = Arc::new(AudioRingBuffer::new(ring_buffer_capacity));
    let speaker_sample_rate = 44100u32;
    let speaker_channels = 1u16;
    let ring_buffer_capacity = speaker_sample_rate as usize * speaker_channels as usize * 5;
    let speaker_rb = Arc::new(AudioRingBuffer::new(ring_buffer_capacity));
    let _output = AudioOutput::new(speaker_rb.clone(), speaker_channels, speaker_sample_rate)?;

    let mut tasks = tokio::task::JoinSet::new();

    let aec_processor = AecProcessor::new(
        mike_rb.clone(),
        clean_rb.clone(),
        aec_tx_ctrl,
        mute_rx_ctrl,
        speaker_rb.clone(),
    );
    tasks.spawn(async move {
        if let Err(e) = aec_processor.run().await {
            eprintln!("AEC Processor error: {}", e);
        }
    });

    let (tx_vad, rx_vad) = mpsc::channel::<Tensor>(100);
    let vad_processor = VadProcessor::new(
        &args.vad_path,
        clean_rb.clone(),
        tx_vad,
        mike_sample_rate as usize,
        mike_channels as usize,
    )?;

    tasks.spawn(async move {
        if let Err(e) = vad_processor.run().await {
            eprintln!("VAD Processor error: {}", e);
        }
    });

    let (tx_asr, rx_asr) = mpsc::channel::<String>(100);
    let asr_processor = AsrProcessor::new(&args.qwen3asr_path, rx_vad, tx_asr)?;

    tasks.spawn(async move {
        if let Err(e) = asr_processor.run().await {
            eprintln!("ASR Processor error: {}", e);
        }
    });

    let (tx_llm, rx_llm) = mpsc::channel::<String>(100);
    let llm_processor = LlmProcessor::new(&args.qwen3_5_path, rx_asr, tx_llm, aec_rx_ctrl2)?;

    tasks.spawn(async move {
        if let Err(e) = llm_processor.run().await {
            eprintln!("LLM Processor error: {}", e);
        }
    });

    let tts_processor = TtsProcessor::new(
        &args.voxcpm_path,
        rx_llm,
        speaker_rb.clone(),
        mute_tx_ctrl,
        aec_rx_ctrl,
    )?;

    tasks.spawn(async move {
        if let Err(e) = tts_processor.run().await {
            eprintln!("TTS Processor error: {}", e);
        }
    });

    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");

    // 取消所有任务
    tasks.abort_all();

    // 等待任务结束
    while let Some(res) = tasks.join_next().await {
        if let Err(e) = res {
            eprintln!("Task error: {:?}", e);
        }
    }

    Ok(())
}
