use async_trait::async_trait;
use log::trace;
use tokio::io::AsyncWriteExt;
use tokio::net::unix::pipe;
use crate::parser::FfmpegInfo;

pub struct FifoOut {
    fifo_sender: pipe::Sender
}

impl FifoOut {
    pub fn new(fifo_output_file: String) -> anyhow::Result<Self> {
        let tx_test = pipe::OpenOptions::new()
            .read_write(true)
            .open_sender(fifo_output_file)?;
        
        Ok(Self {
            fifo_sender: tx_test
        })
    }
}

#[async_trait]
impl crate::modes::ProcessLog for FifoOut {
    async fn process_log(&mut self, ffmpeg_info: FfmpegInfo) -> anyhow::Result<()> {
        trace!("Sending data to fifo out method");
        
        self.fifo_sender.write_all((format!("{}\n", ffmpeg_info.raw_line)).as_bytes()).await?;
        self.fifo_sender.flush().await?;
        
        Ok(())
    }
}