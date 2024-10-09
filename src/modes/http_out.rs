use anyhow::anyhow;
use async_trait::async_trait;
use avro_rs::{Codec, Schema, Writer};
use avro_rs::types::Value;
use reqwest::{Client};
use crate::args::{HttpFormat};
use crate::parser::FfmpegInfo;

pub struct HttpOut {
    http_endpoint: String,
    http_format: HttpFormat,

    avro_schema: Schema
}

impl HttpOut {
    pub fn new(http_endpoint: String, http_format: HttpFormat) -> anyhow::Result<Self> {
        let raw_schema = include_str!("../../resources/avro/ffmpeg_vstat_info.avsc");
        
        Ok(Self {
            http_endpoint,
            http_format,
            avro_schema: Schema::parse_str(raw_schema)?
        })
    }
}

#[async_trait]
impl crate::modes::ProcessLog for HttpOut {
    async fn process_log(&mut self, ffmpeg_info: FfmpegInfo) -> anyhow::Result<()> {
        let request_builder = Client::new().post(self.http_endpoint.clone());

        let request_builder = match self.http_format {
            HttpFormat::Avro => {
                let mut writer = Writer::with_codec(&self.avro_schema, Vec::new(), Codec::Deflate);

                let mut record = avro_rs::types::Record::new(&self.avro_schema).ok_or(anyhow!("Error parsing the avro schema, must be a record"))?;

                record.put("out_file_index", Value::from(ffmpeg_info.out_file_index));
                record.put("out_stream_index",  Value::from(ffmpeg_info.out_stream_index));
                record.put("frame_number",  Value::from(ffmpeg_info.frame_number));
                record.put("frame_quality",  Value::from(ffmpeg_info.frame_quality));
                record.put("packet_size_bytes",  Value::from(ffmpeg_info.packet_size_bytes));
                record.put("stream_size_kbytes",  Value::from(ffmpeg_info.stream_size_kbytes));
                record.put("timestamp",  Value::from(ffmpeg_info.timestamp));
                record.put("picture_type",  Value::from(ffmpeg_info.picture_type));
                record.put("bitrate_kbps",  Value::from(ffmpeg_info.bitrate_kbps));
                record.put("avg_bitrate_kbps",  Value::from(ffmpeg_info.avg_bitrate_kbps));

                writer.append(record)?;
                writer.flush()?;
                let avro_bytes = writer.into_inner()?;

                request_builder
                    .header("Content-Type", "application/octet-stream")
                    .body(avro_bytes)
            }

            HttpFormat::Json => {
                request_builder.json(&ffmpeg_info)
            }
            
            HttpFormat::Bson => {
                let mut doc = bson::Document::new();
                
                let bson_data = bson::to_bson(&ffmpeg_info)?;
                
                doc.insert("ffmpeg_info", bson_data);
                
                let binary_data = bson::to_vec(&doc)?;
                
                request_builder
                    .header("Content-Type", "application/bson")
                    .body(binary_data)
            }
            
            HttpFormat::MsgPack => {
                let msgpack_data = rmp_serde::to_vec(&ffmpeg_info)?;
                
                request_builder
                    .header("Content-Type", "application/x-msgpack")
                    .body(msgpack_data)
            }
        };

        request_builder.send().await?;

        Ok(())
    }
}
