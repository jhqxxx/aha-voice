use crate::pipeline::message::{
    AudioRingBuffer, BroadcastCommandTx, ControlCommand, MpscCommandRx,
};
use anyhow::Result;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub struct AecProcessor {
    aec_tx_ctrl: BroadcastCommandTx,
    mute_rx_ctrl: MpscCommandRx,
    muted: Arc<AtomicBool>,
    mike_rb: Arc<AudioRingBuffer>,
    clean_rb: Arc<AudioRingBuffer>,
    speaker_rb: Arc<AudioRingBuffer>,
}

impl AecProcessor {
    pub fn new(
        mike_rb: Arc<AudioRingBuffer>,
        clean_rb: Arc<AudioRingBuffer>,
        aec_tx_ctrl: BroadcastCommandTx,
        mute_rx_ctrl: MpscCommandRx,
        speaker_rb: Arc<AudioRingBuffer>,
    ) -> Self {
        let muted = Arc::new(AtomicBool::new(false));
        Self {
            aec_tx_ctrl,
            mute_rx_ctrl,
            muted,
            mike_rb,
            clean_rb,
            speaker_rb,
        }
    }

    #[allow(unused)]
    pub async fn run(mut self) -> Result<()> {
        loop {
            if let Ok(cmd) = self.mute_rx_ctrl.try_recv()
                && let ControlCommand::SetMute(m) = cmd
            {
                println!("AEC recv SetMute: {m} signal");
                self.muted.store(m, Ordering::Relaxed);
            }
            if self.mike_rb.is_empty() {
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                continue;
            } else {
                let buffer_data = self.mike_rb.pop_all();
                // true 时 tts有数据输出 或 扬声器有数据输出时，高能量数据打断输出
                if self.muted.load(Ordering::Relaxed) || !self.speaker_rb.is_empty() {
                    let energy: f32 =
                        buffer_data.iter().map(|&x| x * x).sum::<f32>() / buffer_data.len() as f32;
                    let rms = energy.sqrt();
                    println!("aec muted true rms: {rms}");
                    if rms > 0.1 {
                        println!("AEC Interrupt");
                        if self.muted.load(Ordering::Relaxed) {
                            match self.aec_tx_ctrl.send(ControlCommand::Interrupt) {
                                Ok(_) => {
                                    println!("AEC rms: {rms} send: Interrupt signal");
                                }
                                Err(e) => {
                                    println!("aec send interrupt error: {e}");
                                }
                            }
                            self.muted.store(false, Ordering::Relaxed); // 打断后 mute置为false
                        }
                        self.clean_rb.push_overwrite(&buffer_data);
                    }
                } else {
                    self.clean_rb.push_overwrite(&buffer_data);
                }
            }
        }
        Ok(())
    }
}
