use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Observer, RingBuffer},
};

/// 音频环形缓冲区
pub struct AudioRingBuffer {
    inner: Arc<Mutex<HeapRb<f32>>>,
    // capacity: usize,
}

impl AudioRingBuffer {
    /// 创建指定容量的环形缓冲区
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HeapRb::new(capacity))),
            // capacity,
        }
    }

    /// 向缓冲区写入数据
    pub fn push_overwrite(&self, data: &[f32]) {
        if let Ok(mut rb) = self.inner.lock() {
            rb.push_slice_overwrite(data);
        }
    }

    pub fn len(&self) -> usize {
        if let Ok(rb) = self.inner.lock() {
            rb.occupied_len()
        } else {
            0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&self) {
        if let Ok(mut rb) = self.inner.lock() {
            rb.clear();
        }
    }

    pub fn pop(&self, buffer: &mut [f32]) -> usize {
        let mut num = 0usize;
        if let Ok(mut rb) = self.inner.lock()
            && rb.occupied_len() > 0
        {
            num = rb.pop_slice(buffer);
        }
        num
    }

    pub fn pop_all(&self) -> Vec<f32> {
        if let Ok(mut rb) = self.inner.lock() {
            let len = rb.occupied_len();
            let mut buffer = vec![0.0f32; len];
            let read = rb.pop_slice(&mut buffer);
            buffer.truncate(read);
            buffer
        } else {
            vec![]
        }
    }
}

/// 用户/系统控制命令
#[derive(Debug, Clone, Copy)]
pub enum ControlCommand {
    /// 开始新对话（清空上下文）
    StartDialog,
    /// 停止当前输出（打断）
    Interrupt,
    /// 静音输入/取消静音输入
    SetMute(bool),
    /// 请求当前状态
    QueryStatus,
    /// 退出
    Shutdown,
}

pub type DataTx<T> = Sender<T>;
pub type DataRx<T> = Receiver<T>;

pub type MpscCommandTx = tokio::sync::mpsc::Sender<ControlCommand>;
pub type MpscCommandRx = tokio::sync::mpsc::Receiver<ControlCommand>;

pub type BroadcastCommandTx = tokio::sync::broadcast::Sender<ControlCommand>;
pub type BroadcastCommandRx = tokio::sync::broadcast::Receiver<ControlCommand>;
