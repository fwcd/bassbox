use dsp::{Signal, Frame, DuplexSample};
use dsp::sample::interpolate::{Converter, Linear};
use crate::util::either::Either;
use super::AudioSource;

/// An audio source that automatically converts to
/// a target sample rate.
pub struct Converting<S> where S: AudioSource, <S::Frame as Frame>::Sample: DuplexSample<f64> {
	inner: Either<S, Converter<S, Linear<S::Frame>>>,
	target_sample_hz: f64
}

impl<S> Converting<S> where S: AudioSource, <S::Frame as Frame>::Sample: DuplexSample<f64> {
	pub fn to_sample_hz(target_sample_hz: f64, mut wrapped: S) -> Converting<S> {
		let source_sample_hz = wrapped.sample_hz();
		Converting {
			inner: if target_sample_hz == source_sample_hz {
				// If the sample rates happen to match exactly
				// despite being stored in floating points, we
				// do not need a converter
				Either::Left(wrapped)
			} else {
				let interpolator = Linear::from_source(&mut wrapped);
				Either::Right(Converter::from_hz_to_hz(wrapped, interpolator, source_sample_hz, target_sample_hz))
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
	
	pub fn wrapped_mut(&mut self) -> &mut S {
		match self.inner {
			Either::Left(ref mut wrapped) => wrapped,
			Either::Right(ref mut converter) => converter.source_mut()
		}
	}
}

impl<S> AudioSource for Converting<S> where S: AudioSource, <S::Frame as Frame>::Sample: DuplexSample<f64> {
	fn sample_hz(&self) -> f64 { self.target_sample_hz }
}

impl<S> Signal for Converting<S> where S: AudioSource, <S::Frame as Frame>::Sample: DuplexSample<f64> {
	type Frame = S::Frame;
	
	fn next(&mut self) -> S::Frame { self.inner.next() }
	
	fn is_exhausted(&self) -> bool { self.inner.is_exhausted() }
}
