//! A collection of digital signal processing nodes
//! for use in an audio graph.

use dsp::Node;
use dsp::sample::rate::Converter;
use crate::audioformat::{StandardFrame, empty_standard_frame};

/// An audio processing node.
pub enum DspNode {
	Empty,
	Silence,
	Source { src: Converter<Box<dyn Iterator<Item=StandardFrame> + Send>>, state: PauseState },
}

pub enum PauseState {
	Paused,
	Playing
}

impl Node<StandardFrame> for DspNode {
	fn audio_requested(&mut self, buffer: &mut [StandardFrame], _sample_hz: f64) {
		match *self {
			Self::Empty => (),
			Self::Silence => silence(buffer),
			Self::Source { ref mut src, ref state } => match state {
				PauseState::Playing => {
					for i in 0..buffer.len() {
						buffer[i] = src.next().unwrap_or_else(|| empty_standard_frame());
					}
				},
				PauseState::Paused => silence(buffer)
			}
		}
	}
}

/// Silences the buffer.
fn silence(buffer: &mut [StandardFrame]) {
	for i in 0..buffer.len() {
		buffer[i] = empty_standard_frame();
	}
}
