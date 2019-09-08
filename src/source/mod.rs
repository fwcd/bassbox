//! A collection of audio sources.

pub mod mp3;
pub mod file;
pub mod pausable;

use crate::audioformat::StandardFrame;

/// A source of audio.
pub trait AudioSource: Iterator<Item=StandardFrame> {
	fn sample_hz(&self) -> f64;
}

pub struct EmptySource;

impl AudioSource for EmptySource {
	fn sample_hz(&self) -> f64 { 44_100.0 }
}

impl Iterator for EmptySource {
	type Item = StandardFrame;

	fn next(&mut self) -> Option<StandardFrame> { None }
}

