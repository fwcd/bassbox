use super::AudioSource;
use crate::audioformat::{StandardFrame, STANDARD_CHANNELS};
use dsp::Frame;

/// An MP3 decoder.
pub struct Mp3Source<R> {
	decoder: minimp3::Decoder<R>,
	sample_rate: i32,
	mp3_frame_data: Vec<i16>,
	mp3_frame_offset: usize
}

impl<R> Mp3Source<R> where R: Read {
	fn new(reader: R) -> AudioSource {
		let mut decoder = minimp3::Decoder::new(reader)
		let initial_frame = decoder.next_frame().expect("Could not read initial frame from MP3"); // TODO: Handle errors
		Mp3Source {
			decoder: decoder,
			sample_rate: initial_frame.sample_rate,
			mp3_frame_data: initial_frame.data,
			mp3_frame_offset: 0
		}
	}
}

impl<R> AudioSource for Mp3Source<R> where R: Read {
	fn next(&mut self) -> Option<StandardFrame> {
		let mp3_frame_size = self.mp3_frame_data.len();
		if self.mp3_frame_offset == mp3_frame_size {
			self.frame = match self.decoder.next_frame() {
				Ok(minimp3::Frame { data, .. }) => data, // TODO: Deal with custom channel counts
				Err(minimp3::Error::Eof) => return None,
				Err(e) => panic!("{:?}", e) // TODO: Handle errors
			}
		}
		
		let frame: StandardFrame = [
			self.mp3_frame_data[self.mp3_frame_offset].to_sample(),
			self.mp3_frame_data[self.mp3_frame_offset + 1].to_sample()
		];
		self.mp3_frame_offset += STANDARD_CHANNELS;
		Some(frame)
	}
}
