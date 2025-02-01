# RustCompressor

RustCompressor is a high-performance command-line tool for compressing and decompressing files in multiple formats, including `gzip`, `zstd`, `lz4`, and `zip`. It is designed for speed, efficiency, and ease of use.

## Features
- **Multi-threaded processing** for fast compression and decompression.
- **Progress tracking** with real-time progress bars.
- **Support for multiple compression formats**: `gzip`, `zstd`, `lz4`, and `zip`.
- **Adjustable compression levels** for balancing speed and compression ratio.
- **Pre-built binaries** so users can run the tool without installing Rust.

## Installation

### Using Pre-Built Binaries
You can download the pre-built binaries from the [Releases](https://github.com/haripatel07/RustCompressor/releases) page.

#### Windows
- Download `RustCompressor.exe` and run it from the command line.

#### macOS/Linux
- Download the binary for your platform.
- Make it executable:
  ```bash
  chmod +x RustCompressor
  ```
- Move it to a directory in your `PATH` for easy access:
  ```bash
  sudo mv RustCompressor /usr/local/bin/
  ```

### Building from Source

To build the project from source, ensure you have Rust installed, then run:

```bash
git clone https://github.com/haripatel07/RustCompressor.git
cd RustCompressor
cargo build --release
```

The compiled binary will be located in `target/release/`.

## Usage

### Compress a File

```bash
RustCompressor compress <input_file> <output_file> --format <compression_format> --compression_level <level>
```

Example:
```bash
RustCompressor compress example.txt data_compressed --format lz4 --compression_level 10
```

### Decompress a File

```bash
RustCompressor decompress <input_file> <output_file> --format <compression_format>
```

Example:
```bash
RustCompressor decompress data_compressed.lz4 data_decompressed --format lz4
```

## Supported Formats
- **gzip** (`.gz`)
- **zstd** (`.zst`)
- **lz4** (`.lz4`)
- **zip** (`.zip`)

## Progress Reporting
RustCompressor includes a live progress bar that shows:
- Elapsed time
- Bytes processed
- Estimated time remaining

## Error Handling
- Detects invalid file paths and unsupported formats.
- Provides user-friendly error messages for troubleshooting.

## License
This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Contributions
Contributions are welcome! Feel free to open an issue or submit a pull request.

## Author
Developed by haripatel07(https://github.com/haripatel07).

