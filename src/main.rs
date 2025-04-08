use crate::args::{APP_ARGS, OutputType, ParserMode};
use crate::modes::ProcessLog;
use crate::parser::{LineInfo, ParseInfo};
use ipipe::OnCleanup;
use log::{debug, info, trace, warn};
use std::io::Read;
use std::sync::{Arc, RwLock, mpsc};

mod args;
mod modes;
mod parser;
pub mod regexes;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    info!(
        "Reading from pipe file '{}' using mode '{:?}'",
        APP_ARGS.fifo_file_in, APP_ARGS.parser_mode
    );

    let rx_test = ipipe::Pipe::open(
        std::path::Path::new(&APP_ARGS.fifo_file_in),
        OnCleanup::NoDelete,
    );

    if let Ok(mut rx) = rx_test {
        debug!("Started read process");

        let (channel_tx, channel_rx) = mpsc::sync_channel::<Vec<u8>>(1);

        let _ = std::thread::spawn(move || {
            debug!("Spawn read from fifo tokio thread");

            loop {
                let mut msg = vec![0; 2048];

                if let Ok(readed) = rx.read(&mut msg) {
                    if readed > 0 {
                        let msg_readed_vec = msg[..readed].to_vec();
                        let _ = channel_tx.try_send(msg_readed_vec);
                    }
                }
            }
        });

        let buffer_line_writter: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
        let buffer_line_reader: Arc<RwLock<String>> = Arc::clone(&buffer_line_writter);

        std::thread::spawn(move || {
            debug!("Spawn write from fifo tokio thread");

            let mut incomplete_line = String::new();

            loop {
                match channel_rx.recv() {
                    Ok(msg) => {
                        let msg_string_test = String::from_utf8(msg);

                        if let Ok(msg_string) = msg_string_test {
                            let msg_bytes = msg_string.as_bytes();

                            let mut start = 0;
                            for (i, &byte) in msg_bytes.iter().enumerate() {
                                if byte == b'\n' {
                                    let line = format!(
                                        "{}{}",
                                        incomplete_line,
                                        String::from_utf8_lossy(&msg_bytes[start..i])
                                    );

                                    if APP_ARGS.wait_msec > 0 {
                                        if let Ok(mut w_line) = buffer_line_writter.write() {
                                            *w_line = line;
                                        }
                                    } else {
                                        process_line(line);
                                    }

                                    incomplete_line.clear();
                                    start = i + 1;
                                }
                            }

                            if start < msg_bytes.len() {
                                incomplete_line
                                    .push_str(&String::from_utf8_lossy(&msg_bytes[start..]));
                            }

                            trace!("Writed: '{}'", msg_string);
                        } else {
                            warn!("Ignoring log line: '{:?}'", msg_string_test.err());
                        }
                    }
                    Err(err) => trace!("Error on FIFO channel: {err}"),
                }
            }
        });

        if APP_ARGS.wait_msec > 0 {
            std::thread::spawn(move || {
                loop {
                    if let Ok(line) = buffer_line_reader.read() {
                        if line.len() > 0 {
                            process_line(line.to_string());
                        }
                    }

                    log::debug!("Waiting for process line: {} ms", APP_ARGS.wait_msec);
                    std::thread::sleep(std::time::Duration::from_millis(APP_ARGS.wait_msec as u64));
                }
            });
        }

        loop {
            if let Ok(exists) = std::fs::exists(APP_ARGS.fifo_file_in.clone()) {
                if !exists {
                    warn!("FIFO IN not exists: removed?");
                    // read_task.;
                }
            }

            debug!(
                "Wait for 1s to check again for {} exists.",
                APP_ARGS.fifo_file_in
            );

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    } else {
        anyhow::bail!("Error opening read of tunnel file")
    }
}

fn process_line(line: String) {
    log::debug!("Processing line: {}", line);

    let parser_info_result = match APP_ARGS.parser_mode {
        ParserMode::Raw => Some(LineInfo {
            raw_line: line,
            parse_info: None,
        }),

        ParserMode::FfmpegVstatV2 => {
            let ffmpeg_info_result = parser::parse_ffmpeg_vstat(&line);

            if let Ok(ffmpeg_info) = ffmpeg_info_result {
                Some(LineInfo {
                    raw_line: line,
                    parse_info: Some(ParseInfo::Ffmpeg(Box::new(ffmpeg_info))),
                })
            } else {
                debug!(
                    "Fail parsing ffmpeg vstat line: {}",
                    ffmpeg_info_result.err().unwrap()
                );

                None
            }
        }

        ParserMode::GigaTools => parser::parse_gigatools(&line).map(|gigatool_info| LineInfo {
            raw_line: line,
            parse_info: Some(ParseInfo::GigaTools(Box::new(gigatool_info))),
        }),

        ParserMode::TspContinuity => parser::parse_tsp_continuity(&line).map(|tsp_info| LineInfo {
            raw_line: line,
            parse_info: Some(ParseInfo::TspContinuity(Box::new(tsp_info))),
        }),

        ParserMode::TspHistory => parser::parse_tsp_history(&line).map(|tsp_info| LineInfo {
            raw_line: line,
            parse_info: Some(ParseInfo::TspHistory(Box::new(tsp_info))),
        }),
    };

    if let Some(parser_info) = parser_info_result {
        log::debug!("Sending to out the parsed info");

        match &APP_ARGS.command {
            OutputType::FifoOut(fifo_out_args) => {
                let processor_test = modes::fifo_out::FifoOut::new(&fifo_out_args.fifo_output);

                if let Ok(mut processor) = processor_test {
                    if let Err(e) = processor.process_log(parser_info) {
                        warn!("Error processing line: {}", e.to_string());
                    }
                } else {
                    panic!(
                        "Error creating processor: {}",
                        processor_test.err().unwrap()
                    )
                }
            }
            OutputType::HttpPost(http_args) => {
                let mut processor = modes::http_out::HttpOut::new(
                    http_args.uri_endpoint.clone(),
                    http_args.data_format.clone(),
                );

                if let Err(e) = processor.process_log(parser_info) {
                    warn!("Error processing line: {}", e.to_string());
                }
            }
        };
    }
}
