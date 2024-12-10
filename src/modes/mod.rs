use crate::parser::LineInfo;

pub(crate) mod fifo_out;
pub(crate) mod http_out;

pub(crate) trait ProcessLog: Send {
    fn process_log(&mut self, line_info: LineInfo) -> anyhow::Result<()>;
}