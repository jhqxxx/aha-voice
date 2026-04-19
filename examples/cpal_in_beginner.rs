use std::sync::{Arc, Mutex};

use aha_voice::err_fn;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() {
    // mikephone input
    let audio_data = Arc::new(Mutex::new(Vec::<f32>::new()));
    let audio_data_clone = Arc::clone(&audio_data);
    let host = cpal::default_host(); // alsa
    let input_device = host
        .default_input_device()
        .expect("failed to find input device");
    let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();
    println!("config: {:?}", config);
    let input_stream = input_device
        .build_input_stream(
            &config,
            move |data: &[f32], _| {
                let mut audio_guard = audio_data_clone.lock().unwrap();
                println!("audio_guard------------ len: {}", audio_guard.len());
                audio_guard.extend_from_slice(data);
            },
            err_fn,
            None,
        )
        .unwrap();
    input_stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
    let audio_len = {
        let audio_guard = audio_data.lock().unwrap();
        audio_guard.len()
    };
    println!("audio_len: {}", audio_len);
    drop(input_stream);
}
