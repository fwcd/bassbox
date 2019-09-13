use dsp::sample::rate::Converter;
use either::Either;
use crate::audioformat::StandardFrame;
use super::AudioSource;

/// An audio source that automatically converts to
/// a target sample rate.
pub struct Converting<S> where S: AudioSource {
	inner: Either<S, Converter<S>>,
	target_sample_hz: f64
}

impl<S> Converting<S> where S: AudioSource {
	pub fn to_sample_hz(target_sample_hz: f64, wrapped: S) -> Converting<S> {
		let source_sample_hz = wrapped.sample_hz();
		Converting {
			inner: if target_sample_hz == source_sample_hz {
				// If the sample rates happen to match exactly
				// despite being stored in floating points, we
				// do not need a converter
				Either::Left(wrapped)
			} else {
				Either::Right(Converter::from_hz_to_hz(wrapped, source_sample_hz, target_sample_hz))
			},
			target_sample_hz: target_sample_hz
		}
	}
	
	pub fn wrapped(&self) -> &S {
		match self.inner {
			Either::Left(ref wrapped) => wrapped,
			Either::Right(ref converter) => converter.source()
		}
	}
}

impl<S> AudioSource for Converting<S> where S: AudioSource {
	fn sample_hz(&self) -> f64 { self.target_sample_hz }
}

impl<S> Iterator for Converting<S> where S: AudioSource {
	type Item = StandardFrame;
	
	fn next(&mut self) -> Option<StandardFrame> {
		match self.inner {
			Either::Left(ref mut wrapped) => wrapped.next(),
			Either::Right(ref mut converter) => converter.next()
		}
	}
}
