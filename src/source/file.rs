use std::io::BufReader;
use std::fs::File;
use dsp::Signal;
use crate::audioformat::StandardFrame;
use super::mp3::Mp3Source;
use super::AudioSource;

/// An audio source whose format can automatically
/// be determined from the file's path upon construction.
enum FileFormatSource {
	Mp3(Mp3Source<BufReader<File>>)
	// TODO: Other formats
}

impl FileFormatSource {
	/// Tries to recognize the file's format and
	/// create an appropriate decoder source for it.
	/// Otherwise returns None.
	fn from(file_path: &str) -> Result<FileFormatSource, String> {
		let splittable_path = file_path.clone();
		let splitter = splittable_path.split(".");
		let reader = BufReader::new(File::open(file_path).map_err(|e| format!("{:?}", e).to_owned())?);
		let extension = splitter.last().map_or_else(|| Err("File has no extension"), Ok)?;
		match extension.as_ref() {
			"mp3" => Ok(FileFormatSource::Mp3(Mp3Source::new(reader))),
			// TODO: Other formats
			_ => Err(format!("Unsupported file extension: {}", extension).to_owned())
		}
	}
}

impl AudioSource for FileFormatSource {
	fn sample_hz(&self) -> f64 {
		match *self {
			FileFormatSource::Mp3(ref src) => src.sample_hz()
		}
	}
}

impl Signal for FileFormatSource {
	type Frame = StandardFrame;
	
	fn next(&mut self) -> StandardFrame {
		match *self {
			FileFormatSource::Mp3(ref mut src) => src.next()
		}
	}
	
	fn is_exhausted(&self) -> bool {
		match *self {
			FileFormatSource::Mp3(ref src) => src.is_exhausted()
		}
	}
}

/// A wrapper around a decoder source that maintains
/// a path to the file it was created from.
pub struct FileSource {
	wrapped: FileFormatSource,
	file_path: String
}

impl FileSource {
	pub fn new(file_path: &str) -> Result<FileSource, String> {
		Ok(FileSource {
			wrapped: FileFormatSource::from(file_path)?,
			file_path: file_path.to_owned()
		})
	}
	
	pub fn file_path(&self) -> &str { self.file_path.as_ref() }
}

impl AudioSource for FileSource {
	fn sample_hz(&self) -> f64 { self.wrapped.sample_hz() }
}

impl Signal for FileSource {
	type Frame = StandardFrame;
	
	fn next(&mut self) -> StandardFrame { self.wrapped.next() }
	
	fn is_exhausted(&self) -> bool { self.wrapped.is_exhausted() }
}
