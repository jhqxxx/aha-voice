use anyhow::Result;
use cpal::{
    Device, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::sync::Arc;

use crate::{pipeline::message::AudioRingBuffer, utils::err_fn};

pub struct AudioOutput {
    _stream: Stream,
    _device: Device,
}

impl AudioOutput {
    pub fn new(speaker_rb: Arc<AudioRingBuffer>, channels: u16, sample_rate: u32) -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device");
        // let cpal_config = device.default_input_config()?.config();
        let cpal_config = cpal::StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };
        let stream = device.build_output_stream(
            &cpal_config,
            move |data: &mut [f32], _| {
                let _ = speaker_rb.pop(data);
            },
            err_fn,
            None,
        )?;
        stream.play()?;

        Ok(Self {
            _stream: stream,
            _device: device,
        })
    }
}
