//! A collection of lowpass and highpass filters

use crate::audioformat::{StandardFrame, empty_standard_frame};
use dsp::sample::Frame;
use std::collections::VecDeque;

/// A facility that processes audio on a frame-by-frame
/// basis, possibly maintaining state across calls.
pub trait Filter {
	/// Applies the filter to a single frame.
	fn apply(&mut self, frame: StandardFrame) -> StandardFrame;
}

/// A filter that only lets low frequencies pass through.
pub struct LowpassFilter {
	delayLine: VecDeque<StandardFrame>,
	length: usize
}

impl LowpassFilter {
	pub fn new() -> LowpassFilter {
		let length: usize = 5;
		LowpassFilter { delayLine: VecDeque::with_capacity(length), length: length }
	}
}

impl Filter for LowpassFilter {
	fn apply(&mut self, frame: StandardFrame) -> StandardFrame {
		if self.delayLine.len() == self.length {
			self.delayLine.pop_front();
		}
		self.delayLine.push_back(frame);
		let out = self.delayLine.iter().fold(empty_standard_frame(), |a, b| a.add_amp(*b));
		return out.scale_amp(1.0 / (self.length as f32));
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
	fn apply(&mut self, frame: StandardFrame) -> StandardFrame {
		let out = self.wrapped.apply(frame);
		if self.disabled { frame } else { out }
	}
}

impl<F> Filter for Box<F> where F: Filter + ?Sized {
	fn apply(&mut self, frame: StandardFrame) -> StandardFrame {
		(**self).apply(frame)
	}
}
