use aha::{
    Tensor,
    models::{qwen3_5::generate::Qwen3_5GenerateModel, voxcpm::generate::VoxCPMGenerate},
};
use aha_voice::utils::{audio_tensor_to_vec, build_mes, err_fn};
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

    /// Path to voxcpm model directory
    #[arg(short = 't', long = "voxcpm-path")]
    voxcpm_path: String,
}

fn main() -> Result<()> {
    // VAD + ASR + LLM + TTS Demo
    let args = Args::parse();
    let audio_data = Arc::new(Mutex::new(Vec::<f32>::new()));
    let audio_data_clone = Arc::clone(&audio_data);
    let vad_data = Arc::new(Mutex::new(Vec::<Tensor>::new()));
    let vad_data_clone = Arc::clone(&vad_data);
    let asr_string = Arc::new(Mutex::new(Vec::<String>::new()));
    let asr_string_clone = Arc::clone(&asr_string);
    let llm_string = Arc::new(Mutex::new(Vec::<String>::new()));
    let llm_string_clone = Arc::clone(&llm_string);
    let tts_data = Arc::new(Mutex::new(Vec::<f32>::new()));
    let tts_data_clone = Arc::clone(&tts_data);
    let tts_data_clone2 = Arc::clone(&tts_data);
    let host = cpal::default_host(); // alsa
    let input_device = host
        .default_input_device()
        .expect("failed to find input device");
    let output_device = host
        .default_output_device()
        .expect("failed to find ouput device");
    let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();
    let channels = config.channels as usize;
    let sample_rate = config.sample_rate as usize;
    let min_num = ((sample_rate * channels) as f32 * 0.5).ceil() as usize;
    std::thread::spawn(move || -> anyhow::Result<()> {
        let input_stream = input_device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                let tts_data_len = {
                    let gard = tts_data_clone2.lock().unwrap();
                    gard.len()
                };
                if tts_data_len == 0 {
                    let mut audio_guard = audio_data_clone.lock().unwrap();
                    audio_guard.extend_from_slice(data);
                }
            },
            err_fn,
            None,
        )?;
        input_stream.play()?;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    std::thread::sleep(std::time::Duration::from_millis(100));
    let mut vad = aha::models::fire_red_vad::vad::FireRedVad::init(&args.vad_path, None, None)?;
    println!("load vad model");
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
            let asr_res = asr.asr_audio(&audio_data)?;
            println!("asr_res: {:?}", asr_res);
            if let Some(text) = asr_res.text {
                let mut asr_string_guard = asr_string_clone.lock().unwrap();
                asr_string_guard.push(text);
            }
        }
    });
    std::thread::sleep(std::time::Duration::from_millis(100));
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
            let speak = asr_vec.join("");
            if speak.chars().count() < 3 {
                continue;
            }
            let mes = build_mes(&speak)?;
            let text = llm.generate_text(mes)?;
            println!("text: {}", text);
            {
                let mut llm_string_guard = llm_string_clone.lock().unwrap();
                llm_string_guard.push(text);
            }
        }
    });
    std::thread::sleep(std::time::Duration::from_millis(100));
    std::thread::spawn(move || -> anyhow::Result<()> {
        let mut tts = VoxCPMGenerate::init(&args.voxcpm_path, None, None)?;
        println!("load tts model");
        let _ = tts.build_prompt_cache(
            "哈喽大家好，我是蒋蒋，aha项目又有新的更新啦，我们添加了".to_string(),
            "file://./assets/jiangjiang.wav".to_string(),
        )?;
        let _ = tts.generate_use_prompt_cache("你好".to_string(), 2, 1024, 10, 2.0, false, 6.0)?;
        println!(" tts model build prompt cache and init");
        // let mut save_i = 0;
        loop {
            let llm_vec = {
                let mut llm_string_guard = llm_string.lock().unwrap();
                let llm_vec = llm_string_guard.clone();
                llm_string_guard.clear();
                llm_vec
            };
            if llm_vec.is_empty() {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            let mut llm_res = llm_vec.join("");
            if llm_res.chars().count() > 100 {
                llm_res = llm_res.chars().take(100).collect();
            }
            println!("tts start: {}", llm_res);
            let audio = tts.generate_use_prompt_cache(llm_res, 2, 1024, 10, 2.0, false, 6.0)?;
            // let _ = save_wav(&audio, &format!("{}.wav", save_i), 44100)?;
            // save_i += 1;
            let audio_vec = audio_tensor_to_vec(&audio)?;
            println!("tts end data add: {}", audio_vec.len());
            {
                let mut tts_data_guard = tts_data_clone.lock().unwrap();
                tts_data_guard.extend_from_slice(&audio_vec);
            }
        }
    });
    std::thread::sleep(std::time::Duration::from_millis(100));
    let output_config = cpal::StreamConfig {
        channels: 1,
        sample_rate: 44100,
        buffer_size: cpal::BufferSize::Default,
    };
    std::thread::spawn(move || -> anyhow::Result<()> {
        let output_stream = output_device.build_output_stream(
            &output_config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let required_samples = output.len();
                let tts_vec = {
                    let mut tts_data_guard = tts_data.lock().unwrap();
                    if tts_data_guard.len() >= required_samples {
                        let tts_vec = (tts_data_guard[0..required_samples]).to_vec();
                        tts_data_guard.drain(0..required_samples);
                        tts_vec
                    } else {
                        let tts_vec = tts_data_guard.clone();
                        tts_data_guard.clear();
                        tts_vec
                    }
                };
                if !tts_vec.is_empty() {
                    let mut tts_iter = tts_vec.into_iter();
                    for sample in output.iter_mut() {
                        *sample = tts_iter.next().unwrap_or(0.0);
                    }
                }
            },
            err_fn,
            None,
        )?;
        output_stream.play()?;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
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
