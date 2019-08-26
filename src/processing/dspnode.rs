//! A collection of digital signal processing nodes
//! for use in an audio graph.

use dsp::Node;
use crate::audioformat::Frame;

/// An audio processing node.
pub enum DspNode {
	Master
}

impl Node<Frame> for DspNode {
	fn audio_requested(&mut self, buffer: &mut [Frame], sample_hz: f64) {
		// TODO
	}
}
