//! Replay I/O for saving and loading.
//!
//! This module provides functions for saving replays to files
//! and loading them back.

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::replay::types::GameReplay;

/// Error type for replay I/O operations.
#[derive(Debug)]
pub enum ReplayIoError {
    /// Failed to read or write file.
    IoError(std::io::Error),
    /// Failed to serialize or deserialize JSON.
    JsonError(serde_json::Error),
    /// File format not recognized.
    UnknownFormat(String),
}

impl std::fmt::Display for ReplayIoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplayIoError::IoError(e) => write!(f, "I/O error: {}", e),
            ReplayIoError::JsonError(e) => write!(f, "JSON error: {}", e),
            ReplayIoError::UnknownFormat(ext) => write!(f, "Unknown file format: {}", ext),
        }
    }
}

impl std::error::Error for ReplayIoError {}

impl From<std::io::Error> for ReplayIoError {
    fn from(err: std::io::Error) -> Self {
        ReplayIoError::IoError(err)
    }
}

impl From<serde_json::Error> for ReplayIoError {
    fn from(err: serde_json::Error) -> Self {
        ReplayIoError::JsonError(err)
    }
}

/// Replay file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayFormat {
    /// Plain JSON (.replay.json)
    Json,
    /// Compressed JSON (.replay.json.gz)
    CompressedJson,
}

impl ReplayFormat {
    /// Detect format from file extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_str()?;

        if name.ends_with(".replay.json.gz") {
            Some(ReplayFormat::CompressedJson)
        } else if name.ends_with(".replay.json") {
            Some(ReplayFormat::Json)
        } else {
            None
        }
    }

    /// Get the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            ReplayFormat::Json => ".replay.json",
            ReplayFormat::CompressedJson => ".replay.json.gz",
        }
    }
}

/// Save a replay to a JSON file.
pub fn save_json<P: AsRef<Path>>(replay: &GameReplay, path: P) -> Result<(), ReplayIoError> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, replay)?;
    Ok(())
}

/// Save a replay to a compressed JSON file.
pub fn save_compressed<P: AsRef<Path>>(replay: &GameReplay, path: P) -> Result<(), ReplayIoError> {
    let file = File::create(path)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let writer = BufWriter::new(encoder);
    serde_json::to_writer(writer, replay)?;
    Ok(())
}

/// Save a replay, auto-detecting format from file extension.
pub fn save<P: AsRef<Path>>(replay: &GameReplay, path: P) -> Result<(), ReplayIoError> {
    let path = path.as_ref();
    let format = ReplayFormat::from_path(path)
        .ok_or_else(|| ReplayIoError::UnknownFormat(
            path.extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_else(|| "none".to_string())
        ))?;

    match format {
        ReplayFormat::Json => save_json(replay, path),
        ReplayFormat::CompressedJson => save_compressed(replay, path),
    }
}

/// Load a replay from a JSON file.
pub fn load_json<P: AsRef<Path>>(path: P) -> Result<GameReplay, ReplayIoError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let replay = serde_json::from_reader(reader)?;
    Ok(replay)
}

/// Load a replay from a compressed JSON file.
pub fn load_compressed<P: AsRef<Path>>(path: P) -> Result<GameReplay, ReplayIoError> {
    let file = File::open(path)?;
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);
    let replay = serde_json::from_reader(reader)?;
    Ok(replay)
}

/// Load a replay, auto-detecting format from file extension.
pub fn load<P: AsRef<Path>>(path: P) -> Result<GameReplay, ReplayIoError> {
    let path = path.as_ref();
    let format = ReplayFormat::from_path(path)
        .ok_or_else(|| ReplayIoError::UnknownFormat(
            path.extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_else(|| "none".to_string())
        ))?;

    match format {
        ReplayFormat::Json => load_json(path),
        ReplayFormat::CompressedJson => load_compressed(path),
    }
}

/// Serialize a replay to JSON string.
pub fn to_json_string(replay: &GameReplay) -> Result<String, ReplayIoError> {
    Ok(serde_json::to_string_pretty(replay)?)
}

/// Serialize a replay to compact JSON string (no whitespace).
pub fn to_json_compact(replay: &GameReplay) -> Result<String, ReplayIoError> {
    Ok(serde_json::to_string(replay)?)
}

/// Deserialize a replay from JSON string.
pub fn from_json_string(json: &str) -> Result<GameReplay, ReplayIoError> {
    Ok(serde_json::from_str(json)?)
}

/// Serialize a replay to compressed bytes.
pub fn to_compressed_bytes(replay: &GameReplay) -> Result<Vec<u8>, ReplayIoError> {
    let json = serde_json::to_string(replay)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(json.as_bytes())?;
    Ok(encoder.finish()?)
}

/// Deserialize a replay from compressed bytes.
pub fn from_compressed_bytes(bytes: &[u8]) -> Result<GameReplay, ReplayIoError> {
    let mut decoder = GzDecoder::new(bytes);
    let mut json = String::new();
    decoder.read_to_string(&mut json)?;
    Ok(serde_json::from_str(&json)?)
}

/// Get estimated file size for a replay in each format.
pub fn estimate_sizes(replay: &GameReplay) -> Result<(usize, usize), ReplayIoError> {
    let json = to_json_compact(replay)?;
    let json_size = json.len();

    let compressed = to_compressed_bytes(replay)?;
    let compressed_size = compressed.len();

    Ok((json_size, compressed_size))
}
