use super::AudioSource;
use crate::audioformat::{StandardFrame, STANDARD_CHANNELS};
use std::io::Read;
use dsp::sample::Sample;

/// An MP3 decoder.
pub struct Mp3Source<R> {
	decoder: minimp3::Decoder<R>,
	sample_rate: f64,
	mp3_frame_data: Vec<i16>,
	mp3_frame_offset: usize
}

impl<R> Mp3Source<R> where R: Read {
	pub fn new(reader: R) -> Mp3Source<R> {
		let mut decoder = minimp3::Decoder::new(reader);
		let initial_frame = decoder.next_frame().expect("Could not read initial frame from MP3"); // TODO: Handle errors
		
		if initial_frame.channels != STANDARD_CHANNELS {
			// TODO
			panic!("MP3 has {} channels, while only exactly {} are supported currently", initial_frame.channels, STANDARD_CHANNELS);
		}

		Mp3Source {
			decoder: decoder,
			sample_rate: initial_frame.sample_rate as f64,
			mp3_frame_data: initial_frame.data,
			mp3_frame_offset: 0
		}
	}
}

impl<R> AudioSource for Mp3Source<R> where R: Read {
	fn sample_hz(&self) -> f64 { self.sample_rate }
}

impl<R> Iterator for Mp3Source<R> where R: Read {
	type Item = StandardFrame;

	fn next(&mut self) -> Option<StandardFrame> {
		let mp3_frame_size = self.mp3_frame_data.len();

		if self.mp3_frame_offset == mp3_frame_size {
			self.mp3_frame_data = match self.decoder.next_frame() {
				Ok(minimp3::Frame { data, .. }) => data, // TODO: Deal with custom channel counts
				Err(minimp3::Error::Eof) => return None,
				Err(e) => panic!("{:?}", e) // TODO: Handle errors
			};
			self.mp3_frame_offset = 0
		}
		
		let frame: StandardFrame = [
			self.mp3_frame_data[self.mp3_frame_offset].to_sample(),
			self.mp3_frame_data[self.mp3_frame_offset + 1].to_sample()
		];

		self.mp3_frame_offset += STANDARD_CHANNELS;
		Some(frame)
	}
}
