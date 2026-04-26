use aha::{Tensor, params::chat::ChatCompletionParameters};
use anyhow::Result;

pub fn build_mes(text: &str) -> Result<ChatCompletionParameters> {
    let message = format!(
        r#"{{
            "model": "qwen3.5",
            "messages": [
                {{
                    "role": "system",
                    "content": "你是蒋小哈，请精简回复"
                }},
                {{
                    "role": "user",
                    "content": "{}"
                }}
            ],
            "max_tokens": 256
        }}"#,
        text.replace('"', "\\\"")
    );
    Ok(serde_json::from_str(&message)?)
}

pub fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on Audio stream: {err}");
}

pub fn audio_tensor_to_vec(audio: &Tensor) -> Result<Vec<f32>> {
    let max = audio.abs()?.max_all()?.to_scalar::<f32>()?;
    let audio = if max > 0.0 {
        audio.affine(1.0 / max as f64, 0.0)?
    } else {
        audio.clone()
    };
    let audio = audio.squeeze(0)?;
    Ok(audio.to_vec1::<f32>()?)
}

pub fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
