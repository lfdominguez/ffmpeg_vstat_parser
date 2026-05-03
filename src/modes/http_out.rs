use crate::args::{APP_ARGS, HttpFormat};
use crate::parser::LineInfo;
use reqwest::blocking::Client;

pub struct HttpOut {
    http_endpoint: String,
    http_format: HttpFormat,
    client: Client,
}

impl HttpOut {
    pub fn new(http_endpoint: String, http_format: HttpFormat) -> Self {
        // Build a single client with HTTP keep-alive enabled so the underlying
        // TCP connection is reused across requests (avoids open/close per send).
        let client = Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .pool_max_idle_per_host(1)
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to build reqwest HTTP client");

        Self {
            http_endpoint,
            http_format,
            client,
        }
    }
}

impl crate::modes::ProcessLog for HttpOut {
    fn process_log(&mut self, line_info: LineInfo) -> anyhow::Result<()> {
        log::debug!("Processing http out");

        let request_builder = self.client.post(self.http_endpoint.clone());

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
