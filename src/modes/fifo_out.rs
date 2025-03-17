use std::io::Write;
use ipipe::OnCleanup;
use log::trace;
use crate::parser::LineInfo;
use anyhow::Context;

pub struct FifoOut {
    fifo_sender: ipipe::Pipe
}

impl FifoOut {
    pub fn new(fifo_output_file: String) -> anyhow::Result<Self> {
        let tx_test = ipipe::Pipe::open(std::path::Path::new(&fifo_output_file), OnCleanup::NoDelete).context("Can't open FIFO out")?;
        
        Ok(Self {
            fifo_sender: tx_test
        })
    }
}

impl crate::modes::ProcessLog for FifoOut {
    fn process_log(&mut self, line_info: LineInfo) -> anyhow::Result<()> {
        trace!("Sending data to fifo out method");
        
        self.fifo_sender.write_all((format!("{}\n", line_info.raw_line)).as_bytes())?;
        
        Ok(())
    }
}