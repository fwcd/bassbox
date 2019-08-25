//! A collection of concrete audio outputs.

pub mod default;

use crate::audioformat::Frame;

/// An audio output capable of playing sound
/// to the user.
pub trait Sink {
	/// Outputs audio through the sink.
	fn output_audio(&self, audio: &[Frame]);
}
