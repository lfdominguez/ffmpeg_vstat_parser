use argh::FromArgs;
use once_cell::sync::Lazy;
use strum_macros::EnumString;

pub static APP_ARGS: Lazy<Args> = Lazy::new(argh::from_env);

#[derive(FromArgs)]
#[argh(description="Parse the vstat ffmpeg format from a PIPE and try to sent to a external service ")]
pub(crate) struct Args {
    #[argh(option, short = 'f', long = "fifo", description = "input file fifo to read from")]
    pub fifo_file_in: String,

    #[argh(option, long = "parser", description = "input log line parser mode")]
    pub parser_mode: ParserMode,

    #[argh(subcommand)]
    pub command: OutputType
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub(crate) enum OutputType {
    FifoOut(OTFifoOut),
    HttpPost(OTHttpPost)
}

#[derive(FromArgs)]
#[argh(subcommand, name = "fifo_out", description = "Output to a fifo file")]
pub(crate) struct OTFifoOut {
    #[argh(positional)]
    pub fifo_output: String
}

#[derive(FromArgs)]
#[argh(subcommand, name = "http_out", description = "Output to a http endpoint as JSON Post")]
pub(crate) struct OTHttpPost {
    #[argh(positional)]
    pub uri_endpoint: String,

    #[argh(option, long = "format", description = "format used for sending as POST data [Json, Avro]")]
    pub data_format: HttpFormat
}

#[derive(EnumString, Clone)]
pub(crate) enum HttpFormat {
    Json,
    MsgPack
}

#[derive(EnumString, Clone, Debug)]
pub(crate) enum ParserMode {
    Raw,
    FfmpegVstatV2,
    GigaTools,
    TspContinuity,
    TspHistory
}