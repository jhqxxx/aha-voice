use aha::utils::get_default_save_dir;
use aha_voice::err_fn;
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
}

fn main() -> Result<()> {
    // VAD
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
    // let min_num =
    //     (16000usize / 1000 * 25) * (sample_rate as f32 / 16000.0).ceil() as usize * channels;
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
    let mut not_enough = false;
    loop {
        if not_enough {
            // println!("audio_vec is not enough");
            std::thread::sleep(std::time::Duration::from_millis(50));
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
        if vad_res.is_none() {
            println!("is not speech");
            std::thread::sleep(std::time::Duration::from_millis(25));
            continue;
        }
        println!("vad_res: {:?}", vad_res);
    }
}
