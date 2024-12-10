use std::io::Read;
use std::sync::mpsc;
use ipipe::OnCleanup;
use log::{debug, info, trace, warn};
use crate::args::{OutputType, ParserMode, APP_ARGS};
use crate::modes::ProcessLog;
use crate::parser::{LineInfo, ParseInfo};

mod args;
mod modes;
mod parser;
pub mod regexes;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    info!("Reading from pipe file '{}' using mode '{:?}'", APP_ARGS.fifo_file_in, APP_ARGS.parser_mode);

    let rx_test = ipipe::Pipe::open(std::path::Path::new(&APP_ARGS.fifo_file_in), OnCleanup::NoDelete);

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
                                    let line = format!("{}{}", incomplete_line, String::from_utf8_lossy(&msg_bytes[start..i]));

                                    let parser_info_result = match APP_ARGS.parser_mode {
                                        ParserMode::Raw => Some(LineInfo {
                                            raw_line: line,
                                            parse_info: None
                                        }),

                                        ParserMode::FfmpegVstatV1 | ParserMode::FfmpegVstatV2 => {
                                            let ffmpeg_info_result = parser::parse_ffmpeg_vstat(&line);
                                            
                                            if let Ok(ffmpeg_info) = ffmpeg_info_result {
                                                Some(LineInfo {
                                                    raw_line: line,
                                                    parse_info: Some(ParseInfo::Ffmpeg(Box::new(ffmpeg_info)))
                                                })
                                            } else {
                                                debug!("Fail parsing ffmpeg vstat line: {}", ffmpeg_info_result.err().unwrap());
                                                
                                                None
                                            }
                                        }
                                        
                                        ParserMode::GigaTools => {
                                            if let Some(gigatool_info) = parser::parse_gigatools(&line) {
                                                Some(LineInfo {
                                                    raw_line: line,
                                                    parse_info: Some(ParseInfo::GigaTools(Box::new(gigatool_info))),
                                                })
                                            } else {
                                                None
                                            }
                                        }
                                        
                                        ParserMode::TspContinuity => {
                                            if let Some(tsp_info) = parser::parse_tsp_continuity(&line) {
                                                Some(LineInfo {
                                                    raw_line: line,
                                                    parse_info: Some(ParseInfo::TspContinuity(Box::new(tsp_info))),
                                                })
                                            } else {
                                                None
                                            }
                                        }

                                        ParserMode::TspHistory => {
                                            if let Some(tsp_info) = parser::parse_tsp_history(&line) {
                                                Some(LineInfo {
                                                    raw_line: line,
                                                    parse_info: Some(ParseInfo::TspHistory(Box::new(tsp_info))),
                                                })
                                            } else {
                                                None
                                            }
                                        }
                                    };

                                    if let Some(parser_info) = parser_info_result {
                                        match &APP_ARGS.command {
                                            OutputType::FifoOut(fifo_out_args) => {
                                                let processor_test = modes::fifo_out::FifoOut::new(fifo_out_args.fifo_output.clone());

                                                if let Ok(mut processor) = processor_test {
                                                    if let Err(e) = processor.process_log(parser_info) {
                                                        warn!("Error processing line: {}", e.to_string())
                                                    }
                                                } else {
                                                    panic!("Error creating processor: {}", processor_test.err().unwrap())
                                                }
                                            }
                                            OutputType::HttpPost(http_args) => {
                                                let processor_test = modes::http_out::HttpOut::new(http_args.uri_endpoint.clone(), http_args.data_format.clone());

                                                if let Ok(mut processor) = processor_test {
                                                    if let Err(e) = processor.process_log(parser_info) {
                                                        warn!("Error processing line: {}", e.to_string())
                                                    }
                                                } else {
                                                    panic!("Error creating processor: {}", processor_test.err().unwrap())
                                                }
                                            }
                                        };

                                    }
                                    
                                    incomplete_line.clear();
                                    start = i + 1;
                                }
                            }

                            if start < msg_bytes.len() {
                                incomplete_line.push_str(&String::from_utf8_lossy(&msg_bytes[start..]));
                            }

                            trace!("Writed: '{}'", msg_string);
                        } else {
                            warn!("Ignoring log line: '{:?}'", msg_string_test.err());
                        }
                    },
                    Err(err) => trace!("Error on FIFO channel: {err}"),
                }
            }
        });

        loop {
            if let Ok(exists) = std::fs::exists(APP_ARGS.fifo_file_in.clone()) {
                if !exists {
                    warn!("FIFO IN not exists: removed?");
                    // read_task.;
                }
            }

            debug!("Wait for 1s to check again for {} exists.", APP_ARGS.fifo_file_in);

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    } else {
        anyhow::bail!("Error opening read of tunnel file")
    }
}
