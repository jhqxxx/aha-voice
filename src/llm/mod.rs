use std::pin::pin;

use aha::models::qwen3_5::generate::Qwen3_5GenerateModel;
use anyhow::Result;
use futures_util::StreamExt;
use regex::Regex;
use tokio::sync::mpsc;

use crate::{
    pipeline::message::{BroadcastCommandRx, ControlCommand},
    utils::build_mes,
};

pub struct LlmProcessor<'a> {
    model: Qwen3_5GenerateModel<'a>,
    rx_asr: mpsc::Receiver<String>,
    tx_llm: mpsc::Sender<String>,
    aec_rx_ctrl: BroadcastCommandRx,
}

impl<'a> LlmProcessor<'a> {
    pub fn new(
        model_path: &str,
        rx_asr: mpsc::Receiver<String>,
        tx_llm: mpsc::Sender<String>,
        aec_rx_ctrl: BroadcastCommandRx,
    ) -> Result<Self> {
        let model = Qwen3_5GenerateModel::init(model_path, None, None)?;
        Ok(Self {
            model,
            rx_asr,
            tx_llm,
            aec_rx_ctrl,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        println!("LLM Processor started.");
        let re = Regex::new(r"\p{P}+$").unwrap(); // 判断字符串是否有标点符号
        let mut cache_string = "".to_string();

        while let Some(asr_string) = self.rx_asr.recv().await {
            while let Ok(cmd) = self.aec_rx_ctrl.try_recv()
                && let ControlCommand::Interrupt = cmd
            {
                println!("LLM get string: {asr_string}, but recv Interrupt signal");
            }
            println!(">>> llm Received string: {}", asr_string);
            let mes = build_mes(&asr_string)?;
            let mut stream = pin!(self.model.generate_stream_text(mes)?);
            while let Some(item) = stream.next().await {
                if let Ok(cmd) = self.aec_rx_ctrl.try_recv()
                    && let ControlCommand::Interrupt = cmd
                {
                    println!("LLM Stream recv Interrupt signal");
                    cache_string = "".to_string();
                    break;
                }
                if let Ok(res) = item {
                    cache_string = cache_string + &res;
                    // 标点符号可停顿，给TTS做生成
                    if re.is_match(&cache_string) {
                        let str = re.replace(&cache_string, "").trim().to_string();
                        // 去掉标点符号后字符大于1
                        if str.chars().count() > 1 {
                            if let Err(e) = self.tx_llm.send(cache_string.clone()).await {
                                eprintln!("llm final send error: {}", e);
                            }
                            cache_string.clear();
                        }
                    }
                }
            }
            // 流结束还有数据
            if !cache_string.trim().is_empty() {
                if let Err(e) = self.tx_llm.send(cache_string.clone()).await {
                    eprintln!("llm final send error: {}", e);
                }
                cache_string.clear();
            }
        }
        println!("LLM Processor stopped.");
        Ok(())
    }
}

impl<'a> Drop for LlmProcessor<'a> {
    fn drop(&mut self) {
        println!("!!! LlmProcessor DROPPED !!!");
    }
}
