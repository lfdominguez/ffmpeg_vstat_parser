use once_cell::sync::Lazy;
use std::any::type_name;
use std::str::FromStr;
use anyhow::anyhow;
use log::trace;
use regex::Captures;
use serde::Serialize;
use crate::args::{ParserMode, APP_ARGS};

#[derive(Serialize)]
pub(crate) struct LineInfo {
    #[serde(skip_serializing)]
    pub raw_line: String,
    
    pub parse_info: Option<ParseInfo>
}

#[derive(Serialize)]
pub(crate) enum ParseInfo {
    Ffmpeg(Box<FfmpegInfo>)
}

#[derive(Serialize)]
pub(crate) struct FfmpegInfo {
    pub out_file_index: Option<i64>,
    pub out_stream_index: Option<i64>,
    pub frame_number: i64,
    pub frame_quality: f64,
    pub packet_size_bytes: i64,
    pub stream_size_kbytes: i64,
    pub timestamp: f64,
    pub picture_type: String,
    pub bitrate_kbps: f64,
    pub avg_bitrate_kbps: f64,
}

// Vstat Version 1
// frame= FRAME q= FRAME_QUALITY PSNR= PSNR f_size= FRAME_SIZE s_size= STREAM_SIZEkB time= TIMESTAMP br= BITRATEkbits/s avg_br= AVERAGE_BITRATEkbits/s

// Vstat Version 2
// out= OUT_FILE_INDEX st= OUT_FILE_STREAM_INDEX frame= FRAME_NUMBER q= FRAME_QUALITYf PSNR= PSNR f_size= FRAME_SIZE s_size= STREAM_SIZEkB time= TIMESTAMP br= BITRATEkbits/s avg_br= AVERAGE_BITRATEkbits/s

static FFMPEG_VSTAT_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    match APP_ARGS.parser_mode {
        ParserMode::FfmpegVstatV1 =>
            regex::Regex::new(r"frame=\s+(?<frame>\d+)\s+q=\s+(?<q>\d+\.\d+)\s+f_size=\s+(?<f_size>\d+)\s+s_size=\s+(?<s_size>\d+)kB\s+time=\s+(?<time>\d+\.\d+)\s+br=\s+(?<br>\d+\.\d+)kbits/s\s+avg_br=\s+(?<avg_br>\d+\.\d+)kbits/s\s+type=\s+(?<type>[a-zA-z]+)").unwrap(),
        
        ParserMode::FfmpegVstatV2 =>
            regex::Regex::new(r"out=\s+(?<out>\d+)\s+st=\s+(?<st>\d+)\s+frame=\s+(?<frame>\d+)\s+q=\s+(?<q>\d+\.\d+)\s+f_size=\s+(?<f_size>\d+)\s+s_size=\s+(?<s_size>\d+)kB\s+time=\s+(?<time>\d+\.\d+)\s+br=\s+(?<br>\d+\.\d+)kbits/s\s+avg_br=\s+(?<avg_br>\d+\.\d+)kbits/s\s+type=\s+(?<type>[a-zA-z]+)").unwrap(),
        
        _ => regex::Regex::new("").unwrap()
    }
});

pub(crate) fn parse_ffmpeg_vstat(log_line: &String) -> anyhow::Result<FfmpegInfo> {
    trace!("Processing line: {log_line}");

    if let Some(vstats_regex_groups) = FFMPEG_VSTAT_REGEX.captures(log_line) {
        let out_file_index = match APP_ARGS.parser_mode {
            ParserMode::FfmpegVstatV1 => None,
            ParserMode::FfmpegVstatV2 => Some(parse_generic_field("out", &vstats_regex_groups)?),
            _ => anyhow::bail!("Incorrect parser mode for ffmpeg vstat")
        };

        let out_stream_index = match APP_ARGS.parser_mode {
            ParserMode::FfmpegVstatV1 => None,
            ParserMode::FfmpegVstatV2 => Some(parse_generic_field("st", &vstats_regex_groups)?),
            _ => anyhow::bail!("Incorrect parser mode for ffmpeg vstat")
        };

        let frame_number = parse_generic_field("frame", &vstats_regex_groups)?;
        let frame_quality = parse_generic_field("q", &vstats_regex_groups)?;
        let packet_size_bytes = parse_generic_field("f_size", &vstats_regex_groups)?;
        let stream_size_kbytes = parse_generic_field("s_size", &vstats_regex_groups)?;
        let timestamp = parse_generic_field("time", &vstats_regex_groups)?;
        let picture_type = parse_string_field("type", &vstats_regex_groups)?;
        let bitrate_kbps = parse_generic_field("br", &vstats_regex_groups)?;
        let avg_bitrate_kbps = parse_generic_field("avg_br", &vstats_regex_groups)?;

        Ok(FfmpegInfo {
            out_file_index,
            out_stream_index,
            frame_number,
            frame_quality,
            packet_size_bytes,
            stream_size_kbytes,
            timestamp,
            picture_type,
            bitrate_kbps,
            avg_bitrate_kbps
        })
    } else {
        anyhow::bail!("cant parse ffmpeg vstat line: {}", log_line)
    }
}

fn parse_generic_field<T: FromStr>(field_name: &str, regex_capture: &Captures) -> anyhow::Result<T> {
    regex_capture.name(field_name).ok_or(anyhow!("fail parsing '{field_name}' field"))?.as_str().parse::<T>().map_err(|_| { anyhow!("fail parsing '{}' on '{field_name}' field",  type_name::<T>()) })
}

fn parse_string_field(field_name: &str, regex_capture: &Captures) -> anyhow::Result<String> {
    Ok(regex_capture.name(field_name).ok_or(anyhow!("fail parsing '{field_name}' field"))?.as_str().to_string())
}
