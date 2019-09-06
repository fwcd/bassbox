//! A collection of lowpass and highpass filters

use crate::audioformat::{StandardFrame, OpsExt, empty_standard_frame};
use dsp::sample::Frame;
use std::collections::VecDeque;

/// A facility that processes audio on a frame-by-frame
/// basis, possibly maintaining state across calls.
pub trait Filter {
	/// Applies the filter to a single frame.
	fn apply(&mut self, input: StandardFrame) -> StandardFrame;
}

/// A simple infinite impulse response (IIR) lowpass filter.
/// 
/// The implementation roughly follows:
/// - https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
pub struct IIRLowpassFilter {
	last_input: StandardFrame,
	last_output: StandardFrame,
	/// The smoothing factor
	alpha: f32
}

impl Filter for IIRLowpassFilter {
	fn apply(&mut self, input: StandardFrame) -> StandardFrame {
		let output = self.last_output.add((input.sub(self.last_input)).scale(self.alpha));
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
	alpha: f32
}

impl Filter for IIRHighpassFilter {
	fn apply(&mut self, input: StandardFrame) -> StandardFrame {
		input // TODO: Implement this
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
pub struct Disableable<F> {
	wrapped: F,
	pub disabled: bool
}

impl<F> Disableable<F> {
	pub fn enabled(wrapped: F) -> Disableable<F> { Disableable { wrapped: wrapped, disabled: false } }

	pub fn disabled(wrapped: F) -> Disableable<F> { Disableable { wrapped: wrapped, disabled: true } }
}

impl<F> Filter for Disableable<F> where F: Filter {
	fn apply(&mut self, input: StandardFrame) -> StandardFrame {
		let output = self.wrapped.apply(input);
		if self.disabled { input } else { output }
	}
}

impl<F> Filter for Box<F> where F: Filter + ?Sized {
	fn apply(&mut self, input: StandardFrame) -> StandardFrame {
		(**self).apply(input)
	}
}
