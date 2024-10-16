# Project Name: Multi Parser

## Overview

This project provides a utility to parse multiple logs types from a named pipe (FIFO) and send the parsed data to an external service. It leverages the Rust programming language to ensure high performance and reliability. By using this tool, users can efficiently manage and analyze FFmpeg log data in real-time for monitoring or further processing.

## Features

- **Efficient Parsing:** Parses logs from a specified FIFO input file (you must take care how to send the data to the FIFO file).
- **Extensible Commands:** Allows for different output types through subcommands.

## Build

To use this utility, you need to have Rust installed. You can install Rust through `rustup` if it's not already installed:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After Rust is installed, clone the repository and build the project:

```sh
git clone https://github.com/lfdominguez/ffmpeg_vstat_parser.git
cd ffmpeg_vstat_parser
cargo build --release
```

## Usage

The tool can be run from the command line with the following format:

```sh
./target/release/ffmpeg_vstat_parser --fifo <input_fifo_file_path> --parser <parser_type> <subcommand>
```

### Supported Arguments

| Argument   | Short | Description                                                 |
|------------|-------|-------------------------------------------------------------|
| `--fifo`   | `-f`  | Input file FIFO to read from                                |
| `--parser` |       | Define what kind of parser to use (see [Parsers](#Parsers)) |
| Subcommand |       | Specifies the operation to be performed                     |

### Parsers

 * `Raw`: Don't parse any line, send a clone of input to the output
 * `FfmpegVstatV1`: Parse ffmpeg vstat v1 information
 * `FfmpegVstatV2`: Parse ffmpeg vstat v2 information

### Subcommands

| Subcommand | Description                                                   | Arguments                                               |
|------------|---------------------------------------------------------------|---------------------------------------------------------|
| `fifo_out` | Output to a FIFO file (this always clone the input line)      | `fifo_output` (positional) - The FIFO output file path  |
| `http_out` | Output to an HTTP endpoint (*not support HTTPS*) as JSON post | `uri_endpoint` (positional) - URI of the HTTP endpoint  |
|            |                                                               | `--format` Format for sending POST data [Json, MsgPack] |

#### MsgPack
For `MessagePack` the definition of the struct (that you must to use to read the binary) is:

```rust
struct FfmpegInfo {
    pub out_file_index: Option<i64>,
    pub out_stream_index: Option<i64>,
    pub frame_number: i64,
    pub frame_quality: f64,
    pub packet_size_bytes: i64,
    pub stream_size_kbytes: i64,
    pub timestamp: f64,
    pub picture_type: String,
    pub bitrate_kbps: f64,
    pub avg_bitrate_kbps: f64
}
```

### Examples

1. **Basic Usage:**

    Suppose you have an FFmpeg vstats log being written to a FIFO file at `/tmp/ffmpeg_fifo`, and you are using vstats version 1 log lines. You could run the utility as follows:

    ```sh
    ./target/release/ffmpeg_vstat_parser --fifo /tmp/ffmpeg_fifo --parser FfmpegVstatV1 fifo_out /tmp/output_fifo
    ```

2. **Using a Different vstats Version:**

    If the vstats version of your log lines is 2, you can specify that accordingly:

    ```sh
    ./target/release/ffmpeg_vstat_parser --fifo /tmp/ffmpeg_fifo --parser FfmpegVstatV2 fifo_out /tmp/output_fifo
    ```

3. **Output to an HTTP Endpoint:**

    To send the parsed vstats logs to an HTTP endpoint in JSON format:

    ```sh
    ./target/release/ffmpeg_vstat_parser --fifo /tmp/ffmpeg_fifo --parser FfmpegVstatV1 http_out http://example.com/endpoint --format Json
    ```

## Contributing

Contributions are welcome! Please fork the repository and create a pull request with your changes. Make sure to update tests as appropriate.

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.

## Contact

For any inquiries or issues, please open an issue in the GitHub repository or contact the maintainers via email.
