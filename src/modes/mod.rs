use async_trait::async_trait;
use crate::parser::LineInfo;

pub(crate) mod fifo_out;
pub(crate) mod http_out;

#[async_trait]
pub(crate) trait ProcessLog: Send {
    async fn process_log(&mut self, line_info: LineInfo) -> anyhow::Result<()>;
}