use aha::{Tensor, models::qwen3_5::generate::Qwen3_5GenerateModel};
use aha_voice::{build_mes, err_fn};
use anyhow::Result;
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

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
}

fn main() -> Result<()> {
    // VAD + ASR + LLM
    let args = Args::parse();
    let audio_data = Arc::new(Mutex::new(Vec::<f32>::new()));
    let audio_data_clone = Arc::clone(&audio_data);
    let host = cpal::default_host(); // alsa
    let input_device = host
        .default_input_device()
        .expect("failed to find input device");
    let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();
    let channels = config.channels as usize;
    let sample_rate = config.sample_rate as usize;
    let min_num = sample_rate * channels;
    std::thread::spawn(move || -> anyhow::Result<()> {
        let input_stream = input_device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                let mut audio_guard = audio_data_clone.lock().unwrap();
                audio_guard.extend_from_slice(data);
            },
            err_fn,
            None,
        )?;
        input_stream.play()?;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    let mut vad = aha::models::fire_red_vad::vad::FireRedVad::init(&args.vad_path, None, None)?;
    println!("load vad model");
    let vad_data = Arc::new(Mutex::new(Vec::<Tensor>::new()));
    let vad_data_clone = Arc::clone(&vad_data);
    let asr_string = Arc::new(Mutex::new(Vec::<String>::new()));
    let asr_string_clone = Arc::clone(&asr_string);
    std::thread::spawn(move || -> anyhow::Result<()> {
        let mut asr = aha::models::qwen3_asr::generate::Qwen3AsrGenerateModel::init(
            &args.qwen3asr_path,
            None,
            None,
        )?;
        println!("load asr model");
        loop {
            let vad_vec = {
                let mut vad_guard = vad_data.lock().unwrap();
                let vad_vec = vad_guard.clone();
                vad_guard.clear();
                vad_vec
            };
            if vad_vec.is_empty() {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            let audio_data = Tensor::cat(&vad_vec, 0)?;
            let asr_res = asr.asr_audio(&audio_data, true)?;
            println!("asr_res: {:?}", asr_res);
            if let Some(text) = asr_res.text {
                let mut asr_string_guard = asr_string_clone.lock().unwrap();
                asr_string_guard.push(text);
            }
        }
    });
    std::thread::spawn(move || -> anyhow::Result<()> {
        let mut llm = Qwen3_5GenerateModel::init_without_visual(&args.qwen3_5_path, None, None)?;
        println!("load llm model");
        loop {
            let asr_vec = {
                let mut asr_guard = asr_string.lock().unwrap();
                let asr_vec = asr_guard.clone();
                asr_guard.clear();
                asr_vec
            };
            if asr_vec.is_empty() {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            let speech = asr_vec.join("");
            let mes = build_mes(&speech)?;
            let text = llm.generate_text(mes)?;
            println!("text: {}", text);
        }
    });
    let mut not_enough = false;
    loop {
        if not_enough {
            // println!("audio_vec is not enough");
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        let audio_vec = {
            let mut audio_guard = audio_data.lock().unwrap();
            if audio_guard.len() < min_num {
                not_enough = true;
                continue;
            }
            let audio_vec = audio_guard.clone();
            audio_guard.clear();
            audio_vec
        };
        not_enough = false;
        let vad_res = vad.detect_frame_f32(audio_vec, channels, Some(sample_rate))?;
        if let Some(vad) = vad_res
            && let Some(audio) = vad.orig_audio
        {
            let mut vad_guard = vad_data_clone.lock().unwrap();
            vad_guard.push(audio);
        } else {
            println!("not speech");
        }
    }
}
