//! A collection of lowpass and highpass filters

use crate::audioformat::{StandardFrame, OpsExt, empty_standard_frame};
use dsp::sample::Frame;
use std::collections::VecDeque;
use std::f32;

/// A facility that processes audio on a frame-by-frame
/// basis, possibly maintaining state across calls.
pub trait Filter {
	type Frame: Frame;

	/// Applies the filter to a single frame.
	fn apply(&mut self, input: Self::Frame) -> Self::Frame;
}

/// Anything that has a cutoff frequency
pub trait CutoffFreq {
	fn cutoff_hz(&self) -> f32;
	
	fn with_cutoff_hz(&self, cutoff_hz: f32) -> Self;
}

/// A simple infinite impulse response (IIR) lowpass filter.
/// 
/// The implementation roughly follows:
/// - https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
pub struct IIRLowpassFilter {
	last_output: StandardFrame,
	/// The smoothing factor
	alpha: f32,
	cutoff_hz: f32,
	sample_hz: f64,
}

impl IIRLowpassFilter {
	pub fn from_cutoff_hz(cutoff_hz: f32, sample_hz: f64) -> IIRLowpassFilter {
		IIRLowpassFilter::new(empty_standard_frame(), cutoff_hz, sample_hz)
	}

	pub fn new(last_output: StandardFrame, cutoff_hz: f32, sample_hz: f64) -> IIRLowpassFilter {
		let x = 2.0 * f32::consts::PI * (cutoff_hz / sample_hz as f32);
		IIRLowpassFilter {
			last_output: last_output,
			alpha: x / (x + 1.0),
			cutoff_hz: cutoff_hz,
			sample_hz: sample_hz
		}
	}
}

impl CutoffFreq for IIRLowpassFilter {
	fn cutoff_hz(&self) -> f32 { self.cutoff_hz }
	
	fn with_cutoff_hz(&self, cutoff_hz: f32) -> IIRLowpassFilter {
		IIRLowpassFilter::new(self.last_output, cutoff_hz, self.sample_hz)
	}
}

impl Filter for IIRLowpassFilter {
	type Frame = StandardFrame;

	fn apply(&mut self, input: StandardFrame) -> StandardFrame {
		// y[i] = y[i - 1] + alpha * (x[i] - y[i - 1])
		let output = self.last_output.add((input.sub(self.last_output)).scale(self.alpha));
		self.last_output = output;
		output
	}
}

/// A simple infinite impulse response (IIR) highpass filter.
/// 
/// The implementation roughly follows:
/// - https://en.wikipedia.org/wiki/High-pass_filter#Discrete-time_realization
pub struct IIRHighpassFilter {
	last_input: StandardFrame,
	last_output: StandardFrame,
	alpha: f32,
	cutoff_hz: f32,
	sample_hz: f64
}

impl IIRHighpassFilter {
	pub fn from_cutoff_hz(cutoff_hz: f32, sample_hz: f64) -> IIRHighpassFilter {
		IIRHighpassFilter::new(empty_standard_frame(), empty_standard_frame(), cutoff_hz, sample_hz)
	}

	pub fn new(last_input: StandardFrame, last_output: StandardFrame, cutoff_hz: f32, sample_hz: f64) -> IIRHighpassFilter {
		let x = 2.0 * f32::consts::PI * (cutoff_hz / sample_hz as f32);
		IIRHighpassFilter {
			last_input: last_input,
			last_output: last_output,
			alpha: 1.0 / (1.0 + x),
			cutoff_hz: cutoff_hz,
			sample_hz: sample_hz
		}
	}
}

impl CutoffFreq for IIRHighpassFilter {
	fn cutoff_hz(&self) -> f32 { self.cutoff_hz }
	
	fn with_cutoff_hz(&self, cutoff_hz: f32) -> IIRHighpassFilter {
		IIRHighpassFilter::new(self.last_input, self.last_output, cutoff_hz, self.sample_hz)
	}
}

impl Filter for IIRHighpassFilter {
	type Frame = StandardFrame;

	fn apply(&mut self, input: StandardFrame) -> StandardFrame {
		// y[i] = alpha * (y[i - 1] + x[i] - x[i - 1])
		let output = self.last_output.add(input).sub(self.last_input).scale(self.alpha);
		self.last_input = input;
		self.last_output = output;
		output
	}
}

/// A basic filter averaging over the last n frames.
/// The moving average filter is a very simple finite
/// impulse response (FIR) filter and a kind of lowpass.
/// For more control over the cutoff frequency, you
/// can use the 'IIRFilter'.
pub struct MovingAverageFilter {
	last: VecDeque<StandardFrame>, // TODO: Migrate to sample::ring_buffer::Bounded once avilable in dsp-chain
	length: usize
}

impl MovingAverageFilter {
	pub fn new(length: usize) -> MovingAverageFilter {
		MovingAverageFilter { last: VecDeque::with_capacity(length), length: length }
	}
}

impl Filter for MovingAverageFilter {
	type Frame = StandardFrame;

	fn apply(&mut self, input: StandardFrame) -> StandardFrame {
		if self.last.len() == self.length {
			self.last.pop_front();
		}
		self.last.push_back(input);
		let output = self.last.iter().fold(empty_standard_frame(), |a, b| a.add_amp(*b));
		return output.scale_amp(1.0 / (self.length as f32));
	}
}

/// A disableable filter that lets the original signal
/// "pass-through" if disabled.
pub struct Disableable<L> {
	pub wrapped: L,
	pub disabled: bool
}

impl<L> Disableable<L> {
	pub fn new(wrapped: L, disabled: bool) -> Disableable<L> { Disableable { wrapped: wrapped, disabled: disabled } }

	pub fn enabled(wrapped: L) -> Disableable<L> { Disableable { wrapped: wrapped, disabled: false } }

	pub fn disabled(wrapped: L) -> Disableable<L> { Disableable { wrapped: wrapped, disabled: true } }
	
	pub fn with<E>(&self, wrapped: E) -> Disableable<E> { Disableable { wrapped: wrapped, disabled: self.disabled } }
}

impl<L> Filter for Disableable<L> where L: Filter {
	type Frame = L::Frame;

	fn apply(&mut self, input: L::Frame) -> L::Frame {
		let output = self.wrapped.apply(input);
		if self.disabled { input } else { output }
	}
}

impl<L> Filter for Box<L> where L: Filter + ?Sized {
	type Frame = L::Frame;

	fn apply(&mut self, input: L::Frame) -> L::Frame {
		(**self).apply(input)
	}
}
