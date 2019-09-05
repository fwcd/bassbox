//! A collection of lowpass and highpass filters

use crate::audioformat::StandardFrame;
use dsp::sample::Frame;

pub trait Filter {
	fn apply(&mut self, frame: StandardFrame) -> StandardFrame;
}

pub struct LowpassFilter {
	last: StandardFrame
}

impl Filter for LowpassFilter {
	fn apply(&mut self, frame: StandardFrame) -> StandardFrame {
		let out = self.last.add_amp(frame);
		self.last = frame;
		return out;
	}
}
