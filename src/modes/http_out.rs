use crate::args::{APP_ARGS, HttpFormat};
use crate::parser::LineInfo;
use reqwest::blocking::Client;

pub struct HttpOut {
    http_endpoint: String,
    http_format: HttpFormat,
}

impl HttpOut {
    pub fn new(http_endpoint: String, http_format: HttpFormat) -> Self {
        Self {
            http_endpoint,
            http_format,
        }
    }
}

impl crate::modes::ProcessLog for HttpOut {
    fn process_log(&mut self, line_info: LineInfo) -> anyhow::Result<()> {
        log::debug!("Processing http out");

        let request_builder = Client::new().post(self.http_endpoint.clone());

        let request_builder = match self.http_format {
            HttpFormat::Json => {
                if let Some(parse_info) = line_info.parse_info {
                    request_builder.json(&parse_info)
                } else {
                    request_builder.json(&format!("{{\"raw\": {} }}", line_info.raw_line))
                }
            }

            HttpFormat::MsgPack => {
                let msgpack_data = rmp_serde::to_vec(&line_info.parse_info)?;

                request_builder
                    .header("Content-Type", "application/x-msgpack")
                    .body(msgpack_data)
            }
        };

        request_builder.send().inspect_err(|_| {
            std::thread::sleep(std::time::Duration::from_secs(1));
        })?;

        Ok(())
    }
}
