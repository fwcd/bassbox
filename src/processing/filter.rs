//! A collection of lowpass and highpass filters

use crate::audioformat::{StandardFrame, empty_standard_frame};
use dsp::sample::Frame;

/// A facility that processes audio on a frame-by-frame
/// basis, possibly maintaining state across calls.
pub trait Filter {
	/// Applies the filter to a single frame.
	fn apply(&mut self, frame: StandardFrame) -> StandardFrame;
}

/// A filter that only lets low frequencies pass through.
pub struct LowpassFilter {
	last: StandardFrame
}

impl LowpassFilter {
	pub fn new() -> LowpassFilter {
		LowpassFilter { last: empty_standard_frame() }
	}
}

impl Filter for LowpassFilter {
	fn apply(&mut self, frame: StandardFrame) -> StandardFrame {
		let out = self.last.add_amp(frame);
		self.last = frame;
		return out.scale_amp(0.5);
	}
}
