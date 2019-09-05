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
	last: VecDeque<StandardFrame>,
	length: usize
}

impl LowpassFilter {
	pub fn new() -> LowpassFilter {
		let length: usize = 5;
		LowpassFilter { last: VecDeque::with_capacity(length), length: length }
	}
}

impl Filter for LowpassFilter {
	fn apply(&mut self, frame: StandardFrame) -> StandardFrame {
		if self.last.len() == self.length {
			self.last.pop_front();
		}
		self.last.push_back(frame);
		let out = self.last.iter().fold(empty_standard_frame(), |a, b| a.add_amp(*b));
		return out.scale_amp(1.0 / (self.length as f32));
	}
}
