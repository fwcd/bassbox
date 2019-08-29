//! A collection of digital signal processing nodes
//! for use in an audio graph.

use dsp::Node;
use dsp::sample::rate::Converter;
use crate::audioformat::StandardFrame;

/// An audio processing node.
pub enum DspNode {
	Empty,
	Source(Converter<Box<dyn Iterator<Item=StandardFrame> + Send>>)
}

impl Node<StandardFrame> for DspNode {
	fn audio_requested(&mut self, buffer: &mut [StandardFrame], _sample_hz: f64) {
		match *self {
			Self::Empty => (),
			Self::Source(ref mut src) => for i in 0..buffer.len() {
				if let Some(frame) = src.next() {
					buffer[i] = frame
				}
			}
		}
	}
}
