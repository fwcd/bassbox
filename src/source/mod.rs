//! A collection of audio sources.

use crate::audioformat::StandardFrame;

/// A source of audio.
pub trait AudioSource: Iterator<Item=StandardFrame> {
	fn sample_rate(&self) -> i32;
}

