use crc32fast::Hasher;
use hex;
use std::cmp::min;
use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
static AUDIOINFO_VERSION: i32 = 0;
static AUDIOINFO_FILETYPE: &str = "audioinfo";
static AUDIOINFO_CREATED_BY: &str = "audioinfo";

#[derive(Debug)]
enum ProcessError {
    NonFlacError,
    FlacLoadError(FlacLoadError),
    NoSamplesFound,
    UnsupportedBitDepth,
}

#[derive(Debug)]
enum FlacLoadError {
    FileOpenError(std::io::Error),
    FlacReaderError(claxon::Error),
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessError::FlacLoadError(err) => write!(f, "Flac load error: {}", err),
            ProcessError::NoSamplesFound => write!(f, "Total samples not found"),
            ProcessError::NonFlacError => write!(f, "Non flac file found"),
            ProcessError::UnsupportedBitDepth => write!(f, "Unsupported bit depth"),
        }
    }
}

impl fmt::Display for FlacLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FlacLoadError::FileOpenError(err) => write!(f, "Error opening file: {}", err),
            FlacLoadError::FlacReaderError(err) => {
                write!(f, "Error creating claxon FLAC reader: {}", err)
            }
        }
    }
}

impl From<FlacLoadError> for ProcessError {
    fn from(err: FlacLoadError) -> Self {
        ProcessError::FlacLoadError(err)
    }
}

impl From<std::io::Error> for FlacLoadError {
    fn from(err: std::io::Error) -> Self {
        FlacLoadError::FileOpenError(err)
    }
}

impl From<claxon::Error> for FlacLoadError {
    fn from(err: claxon::Error) -> Self {
        FlacLoadError::FlacReaderError(err)
    }
}
#[derive(Debug)]
pub struct AudioInfo {
    type_: String,
    version: i32,
    created_by: String,
    summary: Summary,
    files: Vec<AudioFile>,
}
#[derive(Debug)]
struct Summary {
    total_files: usize,
    total_duration: String,
    sample_rate: i32,
    bit_depth: i32,
    channels: i32,
}

#[derive(Debug)]
struct AudioFile {
    file_name: String,
    duration: String,
    total_samples: u64,
    sample_rate: u32,
    bit_depth: u32,
    channels: u32,
    peak_level: f32,
    rms_db_level: f64,
    crc32: String,
    md5: String,
}

impl Default for AudioFile {
    fn default() -> Self {
        Self {
            file_name: String::default(),
            duration: String::default(),
            total_samples: 0,
            sample_rate: 0,
            bit_depth: 0,
            channels: 0,
            peak_level: 0.0,
            rms_db_level: 0.0,
            crc32: String::default(),
            md5: String::default(),
        }
    }
}

impl AudioInfo {
    pub fn generate_audio_info_from_path(path: PathBuf) -> String {
        let songs = Self::walk_dir(&path);
        let total_duration = Self::add_durations(&songs);
        let audio_info = AudioInfo {
            type_: AUDIOINFO_FILETYPE.to_owned(),
            version: AUDIOINFO_VERSION,
            created_by: AUDIOINFO_CREATED_BY.to_owned(),
            summary: Summary {
                total_files: songs.len(),
                total_duration,
                sample_rate: 44100,
                bit_depth: 16,
                channels: 2,
            },
            files: songs,
        };
        audio_info.to_yaml()
    }

    fn walk_dir(dir: &PathBuf) -> Vec<AudioFile> {
        let mut songs: Vec<AudioFile> = Vec::new();
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|f| {
                !f.path()
                    .iter()
                    .any(|s| s.to_str().map(|x| x.starts_with('.')).unwrap_or(false))
            })
        {
            if entry.file_type().is_file() && entry.path().extension() == Some("flac".as_ref()) {
                tracing::debug!("Processing: {:?}", &entry);
                match Self::process_audio_file(&entry) {
                    Ok(audio_file) => {
                        tracing::debug!("{:?}", audio_file);
                        songs.push(audio_file);
                    }
                    Err(ProcessError::NonFlacError) => {
                        tracing::debug!("Non flac file found: {:?}", &entry.path());
                        continue;
                    }
                    Err(err) => {
                        tracing::error!("Error processing file: {}", err);
                        break;
                    }
                }
            }
        }
        songs
    }

    fn process_audio_file(entry: &walkdir::DirEntry) -> Result<AudioFile, ProcessError> {
        if entry.file_type().is_file() && entry.path().extension() == Some("flac".as_ref()) {
            let mut reader = Self::load_flac(entry.path())?;
            let stream_info = reader.streaminfo();
            let total_samples = stream_info.samples.ok_or(ProcessError::NoSamplesFound)?;
            let bit_depth = stream_info.bits_per_sample;
            let duration = total_samples as f32 / stream_info.sample_rate as f32;

            let mut audio_info = AudioFile {
                file_name: entry.file_name().to_string_lossy().to_string(),
                duration: Self::format_duration(duration),
                total_samples,
                sample_rate: stream_info.sample_rate,
                bit_depth,
                channels: stream_info.channels,
                md5: hex::encode(stream_info.md5sum),
                ..Default::default()
            };

            match bit_depth {
                16 => {
                    let samples: Vec<i16> = reader
                        .samples()
                        .map(|sample| sample.unwrap_or(0) as i16)
                        .collect();
                    audio_info.crc32 = Self::generate_crc32_16bit(&samples);
                    audio_info.peak_level = Self::calculate_peak_level_16bit(&samples);
                    audio_info.rms_db_level = Self::calculate_rms_db_level(samples, 16);
                }
                24 => {
                    let samples: Vec<i32> = reader
                        .samples()
                        .map(|sample| sample.unwrap_or(0) as i32)
                        .collect();
                    audio_info.crc32 = Self::generate_crc32_24bit(&samples);
                    audio_info.peak_level = Self::calculate_peak_level_24bit(&samples);
                    audio_info.rms_db_level = Self::calculate_rms_db_level(samples, 24);
                }
                _ => return Err(ProcessError::UnsupportedBitDepth),
            };

            Ok(audio_info)
        } else {
            Err(ProcessError::NonFlacError)
        }
    }

    fn calculate_rms_db_level<T>(samples: Vec<T>, bit_depth: i32) -> f64
    where
        T: Into<f64> + Clone,
    {
        if samples.is_empty() {
            return f64::NEG_INFINITY; // Return -∞ if the samples vector is empty
        }

        let max_amplitude = Self::get_max_amplitude(bit_depth) as f64;
        let squared_sum: f64 = samples
            .iter()
            .map(|sample| {
                let normalized_sample = (*sample).clone().into() / max_amplitude;
                normalized_sample * normalized_sample
            })
            .sum();

        let rms_amplitude = (squared_sum / samples.len() as f64).sqrt();
        let rms_db_level = 20.0 * rms_amplitude.log10();
        (rms_db_level * 100.0).round() / 100.0
    }

    fn get_max_amplitude(bit_depth: i32) -> i32 {
        match bit_depth {
            16 => i16::MAX as i32,
            24 => (1 << 23) - 1, // For 24-bit audio, the maximum value is 2^23 - 1
            // Add other bit depth cases here if needed
            _ => i16::MAX as i32, // Default to 16-bit audio
        }
    }

    fn calculate_peak_level_16bit(samples: &Vec<i16>) -> f32 {
        let max_amplitude = samples.iter().fold(0, |max, &sample| sample.abs().max(max));
        let peak_level = max_amplitude as f32 / i16::MAX as f32;
        peak_level
    }

    fn calculate_peak_level_24bit(samples: &Vec<i32>) -> f32 {
        let max_amplitude = samples.iter().fold(0, |max, &sample| sample.abs().max(max));
        let max_possible_amplitude = Self::get_max_amplitude(24);
        let peak_level = max_amplitude as f32 / max_possible_amplitude as f32;
        (peak_level * 1000000.0).trunc() / 1000000.0
    }

    fn generate_crc32_24bit(samples: &Vec<i32>) -> String {
        let mut crc32 = Hasher::new();

        for sample in samples {
            let sample_24bit = ((*sample as i32) << 8) >> 8;
            let bytes = [
                (sample_24bit & 0xFF) as u8,
                ((sample_24bit >> 8) & 0xFF) as u8,
                ((sample_24bit >> 16) & 0xFF) as u8,
            ];
            crc32.update(&bytes);
        }
        let crc32_hash = crc32.finalize();
        format!("{:08X}", crc32_hash)
    }

    fn generate_crc32_16bit(samples: &Vec<i16>) -> String {
        let mut crc32 = Hasher::new();

        for sample in samples {
            crc32.update(&sample.to_le_bytes());
        }
        let crc32_hash = crc32.finalize();
        format!("{:08X}", crc32_hash)
    }

    fn add_durations(audio_files: &Vec<AudioFile>) -> String {
        let total_duration_secs: f32 = audio_files
            .iter()
            .map(|audio_file| Self::parse_duration(&audio_file.duration))
            .sum();

        Self::format_duration(total_duration_secs)
    }

    fn parse_duration(duration: &str) -> f32 {
        let components: Vec<&str> = duration.split(':').collect();
        let hours: f32 = components.get(0).unwrap_or(&"0").parse().unwrap_or(0.0);
        let minutes: f32 = components.get(1).unwrap_or(&"0").parse().unwrap_or(0.0);
        let seconds: f32 = components.get(2).unwrap_or(&"0").parse().unwrap_or(0.0);
        let milliseconds: f32 = components.get(3).unwrap_or(&"00").parse().unwrap_or(0.0);

        hours * 3600.0 + minutes * 60.0 + seconds + milliseconds / 100.0
    }

    fn format_duration(duration_secs: f32) -> String {
        let hours = (duration_secs / 3600.0) as u32;
        let minutes = ((duration_secs - (hours as f32 * 3600.0)) / 60.0) as u32;
        let seconds = (duration_secs - (hours as f32 * 3600.0) - (minutes as f32 * 60.0)) as u32;

        let milliseconds = (duration_secs.fract() * 100.0 + 0.5) as u32;

        let capped_milliseconds = min(milliseconds, 99);

        format!(
            "{:02}:{:02}:{:02}.{:02}",
            hours, minutes, seconds, capped_milliseconds
        )
    }

    fn load_flac(path: &Path) -> Result<claxon::FlacReader<File>, FlacLoadError> {
        let flac_file = File::open(path)?;
        let reader = claxon::FlacReader::new(flac_file)?;
        Ok(reader)
    }

    fn to_yaml(&self) -> String {
        let mut yaml = String::new();
        yaml.push_str("type: ");
        yaml.push_str(&self.type_);
        yaml.push('\n');

        yaml.push_str("version: ");
        yaml.push_str(&self.version.to_string());
        yaml.push('\n');

        yaml.push_str("created_by: ");
        yaml.push_str(&self.created_by);
        yaml.push('\n');

        yaml.push_str("summary:\n");
        yaml.push_str("  total_files: ");
        yaml.push_str(&self.summary.total_files.to_string());
        yaml.push('\n');

        yaml.push_str("  total_duration: ");
        yaml.push_str(&self.summary.total_duration);
        yaml.push('\n');

        yaml.push_str("  sample_rate: ");
        yaml.push_str(&self.summary.sample_rate.to_string());
        yaml.push('\n');

        yaml.push_str("  bit_depth: ");
        yaml.push_str(&self.summary.bit_depth.to_string());
        yaml.push('\n');

        yaml.push_str("  channels: ");
        yaml.push_str(&self.summary.channels.to_string());
        yaml.push('\n');

        yaml.push_str("files:\n");
        for file in &self.files {
            yaml.push_str("  - file_name: ");
            yaml.push_str(&file.file_name);
            yaml.push('\n');

            yaml.push_str("    duration: ");
            yaml.push_str(&file.duration);
            yaml.push('\n');

            yaml.push_str("    total_samples: ");
            yaml.push_str(&file.total_samples.to_string());
            yaml.push('\n');

            yaml.push_str("    sample_rate: ");
            yaml.push_str(&file.sample_rate.to_string());
            yaml.push('\n');

            yaml.push_str("    bit_depth: ");
            yaml.push_str(&file.bit_depth.to_string());
            yaml.push('\n');

            yaml.push_str("    channels: ");
            yaml.push_str(&file.channels.to_string());
            yaml.push('\n');

            yaml.push_str("    peak_level: ");
            yaml.push_str(&format!("{:.6}", file.peak_level));
            yaml.push('\n');

            yaml.push_str("    rms_db_level: ");
            yaml.push_str(&format!("{:.2}", file.rms_db_level));
            yaml.push('\n');

            yaml.push_str("    crc32: ");
            yaml.push_str(&file.crc32);
            yaml.push('\n');

            yaml.push_str("    md5: ");
            yaml.push_str(&file.md5);
            yaml.push('\n');
            yaml.push('\n');
        }
        yaml
    }
}
