use std::any::type_name;
use std::str::FromStr;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering::Relaxed;
use anyhow::anyhow;
use log::trace;
use regex::Captures;
use serde::Serialize;
use crate::regexes::{FFMPEG_VSTAT_REGEX, TSP_RE_CONTINUITY, TSP_RE_PID_MODE, TSP_RE_TDT};

#[derive(Serialize)]
pub(crate) struct LineInfo {
    #[serde(skip_serializing)]
    pub raw_line: String,
    
    pub parse_info: Option<ParseInfo>
}

#[derive(Serialize)]
pub(crate) enum ParseInfo {
    Ffmpeg(Box<FfmpegInfo>),
    GigaTools(Box<GigaToolsInfo>),
    TspContinuity(Box<TspContinuity>),
    TspHistory(Box<TspHistory>)
}

#[derive(Serialize)]
pub(crate) struct FfmpegInfo {
    pub index: String,
    pub frame_number: i64,
    pub frame_quality: f64,
    pub packet_size_bytes: i64,
    pub stream_size_kbytes: i64,
    pub timestamp: f64,
    pub picture_type: String,
    pub bitrate_kbps: f64,
    pub avg_bitrate_kbps: f64,
}

#[derive(Serialize)]
pub(crate) struct GigaToolsInfo {
    pub delta_plus: i64,
    pub delta_zero: i64,
    pub pcr_delta: i64,
    pub pcr_freq: i64,
    pub lost_sync: i64,
}

#[derive(Serialize)]
pub(crate) struct TspContinuity {
    pub program_pid: String,
    pub missing_count: i64
}

#[derive(Serialize)]
pub(crate) struct TspHistory {
    pub program_pid: Option<String>,

    pub tdt_datetime_ms: Option<i64>,
    pub is_reset: bool,

    pub action: Option<String>,
}

pub(crate) fn parse_ffmpeg_vstat(log_line: &String) -> anyhow::Result<FfmpegInfo> {
    trace!("Processing line: {log_line}");

    if let Some(vstats_regex_groups) = FFMPEG_VSTAT_REGEX.captures(log_line) {
        let out_file_index: String = parse_generic_field("out", &vstats_regex_groups)?;
        let out_stream_index: String = parse_generic_field("st", &vstats_regex_groups)?;
        let frame_number = parse_generic_field("frame", &vstats_regex_groups)?;
        let frame_quality = parse_generic_field("q", &vstats_regex_groups)?;
        let packet_size_bytes = parse_generic_field("f_size", &vstats_regex_groups)?;
        let stream_size_kbytes = parse_generic_field("s_size", &vstats_regex_groups)?;
        let timestamp = parse_generic_field("time", &vstats_regex_groups)?;
        let picture_type = parse_string_field("type", &vstats_regex_groups)?;
        let bitrate_kbps = parse_generic_field("br", &vstats_regex_groups)?;
        let avg_bitrate_kbps = parse_generic_field("avg_br", &vstats_regex_groups)?;

        Ok(FfmpegInfo {
            index: format!("{}:{}", out_file_index, out_stream_index),
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

pub(crate) fn parse_gigatools(log_line: &String) -> Option<GigaToolsInfo> {
    trace!("Processing line: {log_line}");

    let split: Vec<String> = log_line.split(' ').map(|elem| elem.to_string()).collect();

    if log_line.contains("STAT ALERT-count") {
        log::trace!("Gigatool ALERT log: {log_line}");

        let delta_plus = split.get(8).and_then(|str| str.parse().ok()).unwrap_or(0);
        let delta_zero = split.get(9).and_then(|str| str.parse().ok()).unwrap_or(0);
        let pcr_delta = split.get(10).and_then(|str| str.parse().ok()).unwrap_or(0);
        let pcr_freq = split.get(11).and_then(|str| str.parse().ok()).unwrap_or(0);
        let lost_sync = split.get(12).and_then(|str| str.parse().ok()).unwrap_or(0);

        Some(GigaToolsInfo {
            delta_plus,
            delta_zero,
            pcr_delta,
            pcr_freq,
            lost_sync
        })
    } else {
        None
    }
}

pub(crate) fn parse_tsp_continuity(log_line: &str) -> Option<TspContinuity> {
    if let Some(re_continuity_capture) = TSP_RE_CONTINUITY.captures(log_line) {
        if let (Some(program_pid), Some(missing_count)) =
            (re_continuity_capture.name("program_pid"), re_continuity_capture.name("missing_count"))
        {
            let program_pid = program_pid.as_str().to_string();
            let missing_count = missing_count.as_str().parse::<i64>().unwrap_or_default();

            Some (TspContinuity {
                program_pid,
                missing_count
            })
        } else {
            None
        }
    } else {
        None
    }
}

pub(crate) fn parse_tsp_history(log_line: &str) -> Option<TspHistory> {
    static LATEST_TDT_TIMESTAMP: AtomicI64 = AtomicI64::new(0);

    if let Some(re_history_msg) = TSP_RE_CONTINUITY.captures(log_line) {
        if let Some(message) = re_history_msg.name("message") {
            let message = message.as_str();

            if let Some(re_tdt_msg) = TSP_RE_TDT.captures(message) {
                if let Some(datetime) = re_tdt_msg.name("datetime") {
                    let tdt_time: i64 = chrono::DateTime::parse_from_str(datetime.as_str(), "%Y/%m/%d %H:%M:%S").unwrap().timestamp_millis();

                    let latest_timestamp = LATEST_TDT_TIMESTAMP.load(Relaxed);

                    let is_reset = latest_timestamp > 0 && latest_timestamp >= tdt_time;

                    LATEST_TDT_TIMESTAMP.store(tdt_time, Relaxed);

                    Some(TspHistory {
                        program_pid: None,
                        tdt_datetime_ms: Some(tdt_time),
                        is_reset,
                        action: None,
                    })
                } else {
                    None
                }
            } else if let Some(re_pid_mode_msg) = TSP_RE_PID_MODE.captures(message) {
                if let (Some(pid), Some(action)) = (re_pid_mode_msg.name("pid"), re_pid_mode_msg.name("action")) {
                    Some(TspHistory {
                        program_pid: Some(String::from(pid.as_str())),
                        tdt_datetime_ms: None,
                        is_reset: false,
                        action: Some(String::from(action.as_str())),
                    })
                } else {
                    None
                }
            } else {
                log::warn!("Receive unhandled tsp historic log: {message}");
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
