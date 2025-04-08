use reqwest::blocking::{Client};
use crate::args::{HttpFormat, APP_ARGS};
use crate::parser::LineInfo;

pub struct HttpOut {
    http_endpoint: String,
    http_format: HttpFormat,
    wait_msec: Option<i64>,
    last_sent_milliseconds: i64,
    last_fail_milliseconds: Option<i64>
}

impl HttpOut {
    pub fn new(http_endpoint: String, http_format: HttpFormat) -> Self {
        Self {
            http_endpoint,
            wait_msec: if APP_ARGS.wait_msec == 0 { None } else { Some(APP_ARGS.wait_msec) },
            last_sent_milliseconds: chrono::Utc::now().timestamp_millis(),
            http_format,
            last_fail_milliseconds: None
        }
    }
}

impl crate::modes::ProcessLog for HttpOut {
    fn process_log(&mut self, line_info: LineInfo) -> anyhow::Result<()> {

        let current_milliseconds = chrono::Utc::now().timestamp_millis();

        if let Some(wait_msec) = self.wait_msec {
            if current_milliseconds - self.last_sent_milliseconds < wait_msec {
                return Ok(());
            }
        }

        if let Some(last_fail_milliseconds) = self.last_fail_milliseconds {
            if current_milliseconds - last_fail_milliseconds < 10000 {
                return Ok(());
            }
        }

        let request_builder = Client::new().post(self.http_endpoint.clone());

        let request_builder = match self.http_format {
            
            HttpFormat::Json => if let Some(parse_info) = line_info.parse_info {
                request_builder.json(&parse_info)
            } else {
                request_builder.json(&format!("{{\"raw\": {} }}", line_info.raw_line))
            }
            
            HttpFormat::MsgPack => {
                let msgpack_data = rmp_serde::to_vec(&line_info.parse_info)?;
                
                request_builder
                    .header("Content-Type", "application/x-msgpack")
                    .body(msgpack_data)
            }
        };

        request_builder.send()
            .map(|_| {
                self.last_sent_milliseconds = chrono::Utc::now().timestamp_millis();
                self.last_fail_milliseconds = None;
            })
            .inspect_err(|_| {
                self.last_fail_milliseconds = Some(chrono::Utc::now().timestamp_millis());
            })?;

        Ok(())
    }
}
