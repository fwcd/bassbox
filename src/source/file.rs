use std::io::BufReader;
use std::fs::File;
use crate::audioformat::StandardFrame;
use super::mp3::Mp3Source;
use super::AudioSource;

/// An audio source whose format can automatically
/// be determined from the file's path upon construction.
pub enum DecoderSource {
	Mp3(Mp3Source<BufReader<File>>)
	// TODO: Other formats
}

impl DecoderSource {
	/// Tries to recognize the file's format and
	/// create an appropriate decoder source for it.
	/// Otherwise returns None.
	fn from(file_path: &str) -> Option<DecoderSource> {
		let splittable_path = file_path.clone();
		let splitter = splittable_path.split(".");
		let reader = BufReader::new(File::open(file_path).ok()?);
		match splitter.last()?.as_ref() {
			"mp3" => Some(DecoderSource::Mp3(Mp3Source::new(reader))),
			// TODO: Other formats
			_ => None
		}
	}
}

impl AudioSource for DecoderSource {
	fn sample_hz(&self) -> f64 {
		match *self {
			DecoderSource::Mp3(ref src) => src.sample_hz()
		}
	}
}

impl Iterator for DecoderSource {
	type Item = StandardFrame;
	
	fn next(&mut self) -> Option<StandardFrame> {
		match *self {
			DecoderSource::Mp3(ref mut src) => src.next()
		}
	}
}

/// A wrapper around a decoder source that maintains
/// a path to the file it was created from.
pub struct FileSource {
	decoder: DecoderSource,
	file_path: String
}

impl FileSource {
	pub fn new(file_path: String) -> Option<FileSource> {
		Some(FileSource {
			decoder: DecoderSource::from(file_path.as_ref())?,
			file_path: file_path
		})
	}
	
	pub fn file_path(&self) -> &str { self.file_path.as_ref() }
}

impl AudioSource for FileSource {
	fn sample_hz(&self) -> f64 { self.decoder.sample_hz() }
}

impl Iterator for FileSource {
	type Item = StandardFrame;
	
	fn next(&mut self) -> Option<StandardFrame> { self.decoder.next() }
}
