use log::{debug, info, trace, warn};
use crate::args::{OutputType, APP_ARGS};
use tokio::net::unix::pipe;
use tokio::sync::mpsc;
use tokio::io::{AsyncReadExt};

mod args;
mod modes;
mod parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    info!("Reading from pipe file '{}' using version '{}'", APP_ARGS.fifo_file_in, APP_ARGS.vstat_version);

    let rx_test = pipe::OpenOptions::new()
        .read_write(true)
        .open_receiver(APP_ARGS.fifo_file_in.clone());

    if let Ok(mut rx) = rx_test {
        debug!("Started read process");

        let (channel_tx, mut channel_rx) = mpsc::channel::<Vec<u8>>(1);

        tokio::spawn(async move {
            debug!("Spawn write from fifo tokio thread");

            let mut log_processor: Box<dyn modes::ProcessLog> = match &APP_ARGS.command {
                OutputType::FifoOut(fifo_out_args) => {
                    let processor_test = modes::fifo_out::FifoOut::new(fifo_out_args.fifo_output.clone());
                    
                    if let Ok(processor) = processor_test {
                        Box::new(processor)
                    } else {
                        panic!("Error creating processor: {}", processor_test.err().unwrap().to_string())
                    }
                }
                OutputType::HttpPost(http_args) => {
                    let processor_test = modes::http_out::HttpOut::new(http_args.uri_endpoint.clone(), http_args.data_format.clone());

                    if let Ok(processor) = processor_test {
                        Box::new(processor)
                    } else {
                        panic!("Error creating processor: {}", processor_test.err().unwrap().to_string())
                    }
                }
            };
            
            let mut incomplete_line = String::new();

            loop {
                match channel_rx.recv().await {
                    Some(msg) => {
                        let msg_string_test = String::from_utf8(msg);

                        if let Ok(msg_string) = msg_string_test {
                            let msg_bytes = msg_string.as_bytes();

                            let mut start = 0;
                            for (i, &byte) in msg_bytes.iter().enumerate() {
                                if byte == b'\n' {
                                    let line = format!("{}{}", incomplete_line, String::from_utf8_lossy(&msg_bytes[start..i]));

                                    let ffmpeg_vstat_info_test = parser::parse_ffmpeg_vstat(&line);

                                    if let Ok(ffmpeg_vstat_info) = ffmpeg_vstat_info_test {
                                        if let Err(e) = log_processor.process_log(ffmpeg_vstat_info).await {
                                            warn!("Error processing line: {}", e.to_string())
                                        }
                                    } else {
                                        warn!("Not processing line: {}", ffmpeg_vstat_info_test.err().unwrap().to_string())
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
                    None => trace!("Dropping read package"),
                }
            }
        });

        let read_task = tokio::spawn(async move {
            debug!("Spawn read from fifo tokio thread");

            loop {
                let mut msg = vec![0; 2048];

                if let Ok(readed) = rx.read(&mut msg).await {
                    if readed > 0 {
                        let msg_readed_vec = msg[..readed].to_vec();
                        let _ = channel_tx.try_send(msg_readed_vec);
                    }
                }
            }
        });

        loop {
            if let Ok(exists) = tokio::fs::try_exists(APP_ARGS.fifo_file_in.clone()).await {
                if !exists {
                    read_task.abort();
                }
            }

            debug!("Wait for 1s to check again for {} exists.", APP_ARGS.fifo_file_in);

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    } else {
        anyhow::bail!("Error opening read of tunnel file")
    }
}
