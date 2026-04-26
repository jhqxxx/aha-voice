use std::{pin::pin, sync::Arc};

use aha::models::voxcpm_refact::generate::VoxCPMGenerateRefact;
use anyhow::Result;
use futures_util::StreamExt;
use tokio::sync::mpsc;

use crate::{
    pipeline::message::{AudioRingBuffer, BroadcastCommandRx, ControlCommand, MpscCommandTx},
    utils::audio_tensor_to_vec,
};

pub struct TtsProcessor {
    // model: VoxCPMGenerate,
    model: VoxCPMGenerateRefact,
    rx_llm: mpsc::Receiver<String>,
    speaker_rb: Arc<AudioRingBuffer>,
    mute_tx_ctrl: MpscCommandTx,
    aec_rx_ctrl: BroadcastCommandRx,
}

impl TtsProcessor {
    pub fn new(
        model_path: &str,
        rx_llm: mpsc::Receiver<String>,
        speaker_rb: Arc<AudioRingBuffer>,
        mute_tx_ctrl: MpscCommandTx,
        aec_rx_ctrl: BroadcastCommandRx,
    ) -> Result<Self> {
        // let mut model = VoxCPMGenerate::init(model_path, None, None)?;
        let mut model = VoxCPMGenerateRefact::init(model_path, None, None)?;
        model.build_prompt_cache(
            "哈喽大家好，我是蒋蒋".to_string(),
            "file://./assets/jiangjiang.wav".to_string(),
        )?;
        let _ =
            model.generate_use_prompt_cache("你好".to_string(), 2, 1024, 10, 2.0, false, 6.0)?;
        Ok(Self {
            model,
            rx_llm,
            speaker_rb,
            mute_tx_ctrl,
            aec_rx_ctrl,
        })
    }

    #[allow(unused)]
    pub async fn run(mut self) -> Result<()> {
        println!("TTS Processor started.");
        let mut batch_buffer: Vec<f32> = vec![];
        let min_batch_size = 60000;
        loop {
            if let Ok(llm_string) = self.rx_llm.try_recv() {
                if let Ok(cmd) = self.aec_rx_ctrl.try_recv()
                    && let ControlCommand::Interrupt = cmd
                {
                    println!("TTS get llm string but recv Interrupt signal");
                    while self.rx_llm.try_recv().is_ok() {
                        println!("interrupted clear llm_string");
                    }
                    // 打断时也需要取消 Mute
                    let _ = self.mute_tx_ctrl.send(ControlCommand::SetMute(false)).await;
                    continue;
                }
                // 给aec模块发送tts播放信号
                match self.mute_tx_ctrl.send(ControlCommand::SetMute(true)).await {
                    Ok(_) => {
                        println!("TTS SetMute true signal");
                    }
                    Err(e) => {
                        println!("tts send mute true signal error: {e}");
                    }
                }
                println!(">>> TTS Received string: {}", llm_string);
                let mut stream =
                    pin!(self.model.generate_stream_use_prompt_cache(
                        llm_string, 2, 1024, 10, 2.0, false, 6.0
                    )?);
                let mut is_interrupt = false;
                while let Some(item) = stream.next().await {
                    if let Ok(cmd) = self.aec_rx_ctrl.try_recv()
                        && let ControlCommand::Interrupt = cmd
                    {
                        println!("TTS Stream recv Interrupt signal");
                        batch_buffer.clear();
                        self.speaker_rb.clear();
                        is_interrupt = true;
                        break;
                    }
                    match item {
                        Ok(tts) => {
                            let mut audio_vec = audio_tensor_to_vec(&tts)?;
                            batch_buffer.append(&mut audio_vec);
                            // 流式输出数据量达到要求再播放
                            if batch_buffer.len() >= min_batch_size {
                                self.speaker_rb.push_overwrite(&batch_buffer);
                                batch_buffer.clear();
                            }
                        }
                        Err(e) => {
                            eprintln!("TTS Inference Error: {:?}", e);
                        }
                    }
                }
                // 未被打断且有不足量的数据
                if !is_interrupt && !batch_buffer.is_empty() {
                    self.speaker_rb.push_overwrite(&batch_buffer);
                    batch_buffer.clear();
                }
                // 给aec模块发送tts结束播放信号
                match self.mute_tx_ctrl.send(ControlCommand::SetMute(false)).await {
                    Ok(_) => {
                        println!("TTS SetMute false signal");
                    }
                    Err(e) => {
                        println!("tts send mute false signal error: {e}");
                    }
                }
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }
        Ok(())
    }
}

impl Drop for TtsProcessor {
    fn drop(&mut self) {
        println!("!!! TtsProcessor DROPPED !!!");
    }
}
