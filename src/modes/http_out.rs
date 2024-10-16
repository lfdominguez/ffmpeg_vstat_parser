use async_trait::async_trait;
use reqwest::{Client};
use crate::args::{HttpFormat};
use crate::parser::LineInfo;

pub struct HttpOut {
    http_endpoint: String,
    http_format: HttpFormat
}

impl HttpOut {
    pub fn new(http_endpoint: String, http_format: HttpFormat) -> anyhow::Result<Self> {
        Ok(Self {
            http_endpoint,
            http_format
        })
    }
}

#[async_trait]
impl crate::modes::ProcessLog for HttpOut {
    async fn process_log(&mut self, line_info: LineInfo) -> anyhow::Result<()> {
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

        request_builder.send().await?;

        Ok(())
    }
}
