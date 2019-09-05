//! A collection of digital signal processing nodes
//! for use in an audio graph.

pub mod filter;

use dsp::Node;
use dsp::sample::rate::Converter;
use dsp::sample::Frame;
use filter::Filter;
use crate::audioformat::{StandardFrame, empty_standard_frame};

/// An audio processing node which can either be a source
/// or an intermediate node that performs some transformation
/// on the audio buffer's contents.
/// 
/// _Generally_ all intermediate operations that perform modifications
/// on the audio source (like pausing playback) should either
/// be implemented as a parameter of `DspNode::Source` or in an
/// `AudioSource` implementation wrapping another `AudioSource`.
pub enum DspNode {
	Empty,
	Silence,
	Source { src: Converter<Box<dyn Iterator<Item=StandardFrame> + Send>>, state: PauseState },
	Volume(f32),
	Filter { filter: Box<dyn Filter + Send>, enabled: bool }
}

/// A state of playback.
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
			},
			Self::Volume(factor) => dsp::slice::map_in_place(buffer, |frame| frame.scale_amp(factor)),
			Self::Filter { ref mut filter, enabled } => dsp::slice::map_in_place(buffer, |frame| {
				let output = filter.apply(frame);
				if enabled { output } else { frame }
			})
		}
	}
}

/// Silences the buffer.
fn silence(buffer: &mut [StandardFrame]) {
	for i in 0..buffer.len() {
		buffer[i] = empty_standard_frame();
	}
}
