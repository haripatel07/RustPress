use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write, Result};
use std::sync::{Arc, Mutex};
use std::thread;
use indicatif::{ProgressBar, ProgressStyle};
use flate2::{Compression, write::GzEncoder, read::GzDecoder};
use zstd::Decoder as ZstdDecoder;
use lz4::Decoder as Lz4Decoder;
use zip::write::ZipWriter;
use zip::read::ZipArchive;
use zip::CompressionMethod;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "RustCompressor")]
#[command(about = "A CLI tool for compressing and decompressing files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compress {
        input: String,
        output: String,
        #[arg(short, long, default_value = "gzip")]
        format: String,
        #[arg(short, long, default_value = "10")]
        compression_level: u32,
    },
    Decompress {
        input: String,
        output: String,
        #[arg(short, long, default_value = "gzip")]
        format: String,
    },
}

fn get_compressed_extension(format: &str) -> String {
    match format {
        "gzip" => ".gz".to_string(),
        "zstd" => ".zst".to_string(),
        "lz4" => ".lz4".to_string(),
        "zip" => ".zip".to_string(),
        _ => ".unknown".to_string(),
    }
}

fn compress_file(input_path: &str, output_path: &str, format: &str, compression_level: u32) -> Result<()> {
    let input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;
    let metadata = input_file.metadata()?;
    let total_size = metadata.len();

    let reader = Arc::new(Mutex::new(BufReader::new(input_file)));
    let writer = BufWriter::new(output_file);
    let mut buffer = vec![0; 8192];

    let pb = Arc::new(ProgressBar::new(total_size));
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner} [{elapsed_precise}] {wide_bar} {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let pb_clone = Arc::clone(&pb);
    match format {
        "gzip" => {
            let mut encoder = GzEncoder::new(writer, Compression::new(compression_level as u32));
            let handle = thread::spawn(move || {
                let mut reader = reader.lock().unwrap();
                while let Ok(bytes_read) = reader.read(&mut buffer) {
                    if bytes_read == 0 { break; }
                    encoder.write_all(&buffer[..bytes_read]).unwrap();
                    pb_clone.inc(bytes_read as u64);
                }
                let _ = encoder.finish();
                Ok::<(), std::io::Error>(())  
            });
            handle.join().unwrap().unwrap();  
        }
        "zstd" => {
            let mut encoder = zstd::Encoder::new(writer, compression_level as i32)?;
            let handle = thread::spawn(move || -> Result<()> {
                let mut reader = reader.lock().unwrap();
                while let Ok(bytes_read) = reader.read(&mut buffer) {
                    if bytes_read == 0 { break; }
                    encoder.write_all(&buffer[..bytes_read]).unwrap();
                    pb_clone.inc(bytes_read as u64);
                }
                encoder.finish()?;
                Ok(())
            });
            handle.join().unwrap().unwrap();  
        }
        "lz4" => {
            let mut encoder = lz4::EncoderBuilder::new().level(compression_level as u32).build(writer)?;
            let handle = thread::spawn(move || {
                let mut reader = reader.lock().unwrap();
                while let Ok(bytes_read) = reader.read(&mut buffer) {
                    if bytes_read == 0 { break; }
                    encoder.write_all(&buffer[..bytes_read]).unwrap();
                    pb_clone.inc(bytes_read as u64);
                }
                let _ = encoder.finish().0;
                Ok::<(), std::io::Error>(())  
            });
            handle.join().unwrap().unwrap();  
        }
        "zip" => {
            let mut zip = ZipWriter::new(writer);
            zip.start_file(input_path, zip::write::FileOptions::default().compression_method(CompressionMethod::Deflated))?;
            let handle = thread::spawn(move || {
                let mut reader = reader.lock().unwrap();
                while let Ok(bytes_read) = reader.read(&mut buffer) {
                    if bytes_read == 0 { break; }
                    zip.write_all(&buffer[..bytes_read]).unwrap();
                    pb_clone.inc(bytes_read as u64);
                }
                let _ = zip.finish();
                Ok::<(), std::io::Error>(())  
            });
            handle.join().unwrap().unwrap();  
        }
        _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unsupported format")),
    }
    pb.finish_with_message("Compression complete");
    Ok(())
}

fn decompress_file(input_path: &str, output_path: &str, format: &str) -> Result<()> {
    let input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;
    let metadata = input_file.metadata()?;
    let total_size = metadata.len();

    let reader = BufReader::new(input_file);  // No need for Mutex or Arc
    let mut writer = BufWriter::new(output_file);  
    let mut buffer = vec![0; 8192];

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner} [{elapsed_precise}] {wide_bar} {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    match format {
        "gzip" => {
            let mut decoder = GzDecoder::new(reader);
            while let Ok(bytes_read) = decoder.read(&mut buffer) {
                if bytes_read == 0 { break; }
                writer.write_all(&buffer[..bytes_read])?;
                pb.inc(bytes_read as u64);
            }
        }
        "zstd" => {
            let mut decoder = ZstdDecoder::new(reader)?;
            while let Ok(bytes_read) = decoder.read(&mut buffer) {
                if bytes_read == 0 { break; }
                writer.write_all(&buffer[..bytes_read])?;
                pb.inc(bytes_read as u64);
            }
        }
        "lz4" => {
            let mut decoder = Lz4Decoder::new(reader)?;
            while let Ok(bytes_read) = decoder.read(&mut buffer) {
                if bytes_read == 0 { break; }
                writer.write_all(&buffer[..bytes_read])?;
                pb.inc(bytes_read as u64);
            }
        }
        "zip" => {
            let mut archive = ZipArchive::new(reader)?;
            let mut file = archive.by_index(0)?;  
            while let Ok(bytes_read) = file.read(&mut buffer) {
                if bytes_read == 0 { break; }
                writer.write_all(&buffer[..bytes_read])?;
                pb.inc(bytes_read as u64);
            }
        }
        _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unsupported format")),
    }

    pb.finish_with_message("Decompression complete");
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Compress { input, output, format, compression_level } => {
            let output_with_ext = format!("{}{}", output, get_compressed_extension(&format));
            compress_file(&input, &output_with_ext, &format, compression_level).unwrap();
        }
        Commands::Decompress { input, output, format } => {
            decompress_file(&input, &output, &format).unwrap();
        }
    }
}