use aha::{Tensor, models::qwen3_asr::generate::Qwen3AsrGenerateModel};
use anyhow::Result;
use tokio::sync::mpsc;

pub struct AsrProcessor<'a> {
    model: Qwen3AsrGenerateModel<'a>,
    rx_vad: mpsc::Receiver<Tensor>,
    tx_asr: mpsc::Sender<String>,
}

impl<'a> AsrProcessor<'a> {
    pub fn new(
        model_path: &str,
        rx_vad: mpsc::Receiver<Tensor>,
        tx_asr: mpsc::Sender<String>,
    ) -> Result<Self> {
        let model = Qwen3AsrGenerateModel::init(model_path, None, None)?;
        Ok(Self {
            model,
            rx_vad,
            tx_asr,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        println!("ASR Processor started.");
        let mut cache_string = "".to_string();

        while let Some(audio_tensor) = self.rx_vad.recv().await {
            println!(">>> ASR Received Tensor: {:?}", audio_tensor.shape());
            match self.model.asr_audio(&audio_tensor) {
                Ok(res) => {
                    if let Some(text) = res.text
                        && !text.is_empty()
                    {
                        println!("ASR Result: {}", text);
                        cache_string = cache_string + &text;
                        match self.tx_asr.try_send(cache_string.clone()) {
                            Ok(()) => {
                                cache_string = "".to_string();
                            }
                            Err(e) => {
                                eprintln!("ASR send error: {}. cache: {}", e, cache_string);
                            }
                        }
                    } else {
                        println!("ASR returned no text.");
                    }
                }
                Err(e) => {
                    eprintln!("ASR Inference Error: {:?}", e);
                }
            }
        }
        println!("ASR Processor stopped.");
        Ok(())
    }
}

impl<'a> Drop for AsrProcessor<'a> {
    fn drop(&mut self) {
        println!("!!! AsrProcessor DROPPED !!!");
    }
}
