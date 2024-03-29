//! A collection of digital signal processing nodes
//! for use in an audio graph.

pub mod filter;

use dsp::{Signal, Node};
use dsp::sample::Frame;
use filter::{Filter, MovingAverageFilter, IIRLowpassFilter, IIRHighpassFilter, Disableable};
use crate::source::{AudioSource, file::FileSource, command::CommandSource, conv::Converting, pausable::Pausable};
use crate::audioformat::StandardFrame;
use crate::util::empty::Empty;

/// An audio processing node which can either be a source
/// or an intermediate node that performs some transformation
/// on the audio buffer's contents.
/// 
/// _Generally_ all intermediate operations that perform modifications
/// on the audio source (like pausing playback) should either
/// be implemented as an `AudioSource` decorator.
pub enum DspNode {
	Empty,
	Silence,
	Volume(f32),
	File(Pausable<Converting<FileSource>>),
	Command(Pausable<Converting<CommandSource>>),
	DynSource(Box<dyn AudioSource<Frame=StandardFrame> + Send>),
	MovingAverage(Disableable<MovingAverageFilter>),
	IIRLowpass(Disableable<IIRLowpassFilter>),
	IIRHighpass(Disableable<IIRHighpassFilter>),
	DynFilter(Box<dyn Filter<Frame=StandardFrame> + Send>)
}

impl Node<StandardFrame> for DspNode {
	fn audio_requested(&mut self, buffer: &mut [StandardFrame], _sample_hz: f64) {
		match *self {
			Self::Empty => (),
			Self::Silence => silence(buffer),
			Self::Volume(factor) => dsp::slice::map_in_place(buffer, |frame| frame.scale_amp(factor)),
			// Static source implementations to avoid boxing
			Self::File(ref mut source) => read_signal_into(buffer, source),
			Self::Command(ref mut source) => read_signal_into(buffer, source),
			Self::DynSource(ref mut source) => read_signal_into(buffer, source),
			// Static filter implementations to avoid boxing
			Self::MovingAverage(ref mut filter) => apply_filter(buffer, filter),
			Self::IIRLowpass(ref mut filter) => apply_filter(buffer, filter),
			Self::IIRHighpass(ref mut filter) => apply_filter(buffer, filter),
			Self::DynFilter(ref mut filter) => apply_filter(buffer, filter)
		}
	}
}

impl Empty for DspNode {
	fn empty() -> Self { Self::Empty }
}

pub fn read_signal_into<S, F>(buffer: &mut [F], source: &mut S) where S: Signal<Frame=F>, F: Frame {
	for i in 0..buffer.len() {
		buffer[i] = source.next();
	}
}

pub fn apply_filter<L, F>(buffer: &mut [F], filter: &mut L) where L: Filter<Frame=F>, F: Frame {
	for frame in buffer {
		*frame = filter.apply(*frame);
	}
}

/// Silences the buffer.
fn silence<F>(buffer: &mut [F]) where F: Frame {
	for i in 0..buffer.len() {
		buffer[i] = F::equilibrium();
	}
}
