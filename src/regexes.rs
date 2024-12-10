use once_cell::sync::Lazy;
use crate::args::{ParserMode, APP_ARGS};

pub static FFMPEG_VSTAT_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    match APP_ARGS.parser_mode {
        ParserMode::FfmpegVstatV1 =>
            regex::Regex::new(r"frame=\s+(?<frame>\d+)\s+q=\s+(?<q>\d+\.\d+)\s+f_size=\s+(?<f_size>\d+)\s+s_size=\s+(?<s_size>\d+)kB\s+time=\s+(?<time>\d+\.\d+)\s+br=\s+(?<br>\d+\.\d+)kbits/s\s+avg_br=\s+(?<avg_br>\d+\.\d+)kbits/s\s+type=\s+(?<type>[a-zA-z]+)").unwrap(),

        ParserMode::FfmpegVstatV2 =>
            regex::Regex::new(r"out=\s+(?<out>\d+)\s+st=\s+(?<st>\d+)\s+frame=\s+(?<frame>\d+)\s+q=\s+(?<q>\d+\.\d+)\s+f_size=\s+(?<f_size>\d+)\s+s_size=\s+(?<s_size>\d+)[kK]i?B\s+time=\s+(?<time>\d+\.\d+)\s+br=\s+(?<br>\d+\.\d+)kbits/s\s+avg_br=\s+(?<avg_br>\d+\.\d+)kbits/s\s+type=\s+(?<type>[a-zA-z]+)").unwrap(),

        _ => regex::Regex::new("").unwrap()
    }
});

pub static TSP_RE_CONTINUITY: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"continuity:.+ PID: (?<program_pid>.+), missing (?<missing_count>\d+) packet").unwrap()
});

pub static TSP_RE_HISTORY: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"\* history: (?<packet_number>\d+): (?<message>.+)").unwrap()
});

pub static TSP_RE_TDT: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"TDT: (?<datetime>[0-9]{4}/[0-9]{2}/[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2})").unwrap()
});

pub static TSP_RE_PID_MODE: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"PID \d+ \((?<pid>0x\d+)\) (?<action>[^,]+)").unwrap()
});