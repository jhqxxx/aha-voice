use aha_voice::utils::err_fn;
use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

fn main() -> Result<()> {
    // speaker out
    let wav_path = "./assets/jiangjiang.wav";
    let mut reader = hound::WavReader::open(wav_path)?;
    let spec = reader.spec();
    println!("WAV file specs: {:?}", spec);

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            // 将整数样本转换为浮点数 [-1.0, 1.0]
            match spec.bits_per_sample {
                8 => reader
                    .samples::<i8>()
                    .map(|s| s.map(|sample| sample as f32 / i8::MAX as f32))
                    .collect::<Result<Vec<_>, _>>()?,
                16 => reader
                    .samples::<i16>()
                    .map(|s| s.map(|sample| sample as f32 / i16::MAX as f32))
                    .collect::<Result<Vec<_>, _>>()?,
                24 => reader
                    .samples::<i32>()
                    .map(|s| s.map(|sample| sample as f32 / i32::MAX as f32))
                    .collect::<Result<Vec<_>, _>>()?,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported bit depth: {}",
                        spec.bits_per_sample
                    ));
                }
            }
        }
        hound::SampleFormat::Float => reader.into_samples::<f32>().map(|s| s.unwrap()).collect(),
    };
    let samples_len = samples.len() as f32;
    let host = cpal::default_host(); // alsa
    let out_device = host
        .default_output_device()
        .expect("failed to find output device");
    println!("Output device: {}", out_device.id().unwrap());

    let config = cpal::StreamConfig {
        channels: spec.channels,
        sample_rate: spec.sample_rate,
        buffer_size: cpal::BufferSize::Default,
    };
    let remaining_samples = Arc::new(Mutex::new(samples.into_iter()));
    let stream = out_device.build_output_stream(
        &config,
        move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut samples_iter = remaining_samples.lock().unwrap();

            for sample in output.iter_mut() {
                *sample = samples_iter.next().unwrap_or(0.0);
            }
        },
        err_fn,
        None,
    )?;
    stream.play()?;
    let duration = std::time::Duration::from_secs_f32(
        samples_len / (spec.sample_rate as f32 * spec.channels as f32),
    );
    std::thread::sleep(duration);
    Ok(())
}
