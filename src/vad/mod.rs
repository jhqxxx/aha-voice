use std::sync::Arc;

use aha::{Tensor, models::fire_red_vad::vad::FireRedVad};
use anyhow::Result;
use tokio::sync::mpsc;

use crate::pipeline::message::AudioRingBuffer;

pub struct VadProcessor {
    model: FireRedVad,
    audio_rb: Arc<AudioRingBuffer>,
    tx_vad: mpsc::Sender<Tensor>,
    mike_sample_rate: usize,
    mike_channels: usize,
}

impl VadProcessor {
    pub fn new(
        model_path: &str,
        audio_rb: Arc<AudioRingBuffer>,
        tx_vad: mpsc::Sender<Tensor>,
        mike_sample_rate: usize,
        mike_channels: usize,
    ) -> Result<Self> {
        let model = FireRedVad::init(model_path, None, None)?;
        Ok(Self {
            model,
            audio_rb,
            tx_vad,
            mike_sample_rate,
            mike_channels,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        println!("VAD Processor started.");
        let window_size_samples =
            ((self.mike_sample_rate * self.mike_channels) as f32 * 0.1).ceil() as usize;
        loop {
            if self.audio_rb.len() < window_size_samples {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                continue;
            }
            let buffer_data = self.audio_rb.pop_all();
            let vad_res = self.model.detect_frame_f32(
                buffer_data,
                self.mike_channels,
                Some(self.mike_sample_rate),
            )?;
            if let Some(vad_res) = vad_res
                && let Some(audio) = vad_res.orig_audio
            {
                println!("vad detect speech audio len: {:?}", audio);
                if self.tx_vad.send(audio).await.is_err() {
                    eprintln!("VAD Receiver dropped, stopping VAD processor.");
                    break;
                }
            }
        }
        Ok(())
    }
}
