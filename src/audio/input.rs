use anyhow::Result;
use cpal::{
    Device, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use std::sync::Arc;

use crate::{pipeline::message::AudioRingBuffer, utils::err_fn};

pub struct AudioInput {
    _stream: Stream,
    _device: Device,
}

impl AudioInput {
    pub fn new(audio_rb: Arc<AudioRingBuffer>, channels: u16, sample_rate: u32) -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("no input device");
        let cpal_config = cpal::StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };
        let stream = device.build_input_stream(
            &cpal_config,
            move |data: &[f32], _| {
                audio_rb.push_overwrite(data);
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
