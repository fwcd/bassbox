//! A collection of digital signal processing nodes
//! for use in an audio graph.

pub mod filter;

use dsp::Node;
use dsp::sample::rate::Converter;
use dsp::sample::Frame;
use filter::{Filter, MovingAverageFilter, IIRLowpassFilter, IIRHighpassFilter, Disableable};
use crate::source::{AudioSource, file::FileSource};
use crate::audioformat::{StandardFrame, empty_standard_frame};

/// An audio processing node which can either be a source
/// or an intermediate node that performs some transformation
/// on the audio buffer's contents.
/// 
/// _Generally_ all intermediate operations that perform modifications
/// on the audio source (like pausing playback) should either
/// be implemented as a parameter of `DspNode::DynSource` or in an
/// `AudioSource` implementation wrapping another `AudioSource`.
pub enum DspNode {
	Empty,
	Silence,
	DynSource { src: Converter<Box<dyn Iterator<Item=StandardFrame> + Send>>, state: PauseState },
	Volume(f32),
	MovingAverage(Disableable<MovingAverageFilter>),
	IIRLowpass(Disableable<IIRLowpassFilter>),
	IIRHighpass(Disableable<IIRHighpassFilter>),
	DynFilter(Box<dyn Filter + Send>)
}

/// A state of playback.
pub enum PauseState {
	Paused,
	Playing
}

impl Node<StandardFrame> for DspNode {
	fn audio_requested(&mut self, buffer: &mut [StandardFrame], _sample_hz: f64) {
		match *self {
			DspNode::Empty => (),
			DspNode::Silence => silence(buffer),
			DspNode::DynSource { ref mut src, ref state } => match state {
				PauseState::Playing => {
					for i in 0..buffer.len() {
						buffer[i] = src.next().unwrap_or_else(|| empty_standard_frame());
					}
				},
				PauseState::Paused => silence(buffer)
			},
			DspNode::Volume(factor) => dsp::slice::map_in_place(buffer, |frame| frame.scale_amp(factor)),
			// Static filter implementations to avoid boxing:
			DspNode::MovingAverage(ref mut filter) => apply_filter(buffer, filter),
			DspNode::IIRLowpass(ref mut filter) => apply_filter(buffer, filter),
			DspNode::IIRHighpass(ref mut filter) => apply_filter(buffer, filter),
			DspNode::DynFilter(ref mut filter) => apply_filter(buffer, filter)
		}
	}
}

fn apply_filter<F>(buffer: &mut [StandardFrame], filter: &mut F) where F: Filter {
	for frame in buffer {
		*frame = filter.apply(*frame);
	}
}

/// Silences the buffer.
fn silence(buffer: &mut [StandardFrame]) {
	for i in 0..buffer.len() {
		buffer[i] = empty_standard_frame();
	}
}
