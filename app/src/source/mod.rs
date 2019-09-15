//! A collection of audio sources.

pub mod mp3;
pub mod file;
pub mod command;
pub mod pausable;
pub mod conv;

use std::marker::PhantomData;
use dsp::{Signal, Frame};
use crate::util::either::Either;

/// A source of audio.
pub trait AudioSource: Signal {
	/// The output sample rate of this
	/// audio source in Hertz
	fn sample_hz(&self) -> f64;
}

impl<L, R, F> AudioSource for Either<L, R> where L: AudioSource<Frame=F>, R: AudioSource<Frame=F>, F: Frame {
	fn sample_hz(&self) -> f64 {
		match *self {
			Either::Left(ref wrapped) => wrapped.sample_hz(),
			Either::Right(ref wrapped) => wrapped.sample_hz()
		}
	}
}

impl<L, R, F> Signal for Either<L, R> where L: Signal<Frame=F>, R: Signal<Frame=F>, F: Frame {
	type Frame = F;
	
	fn next(&mut self) -> F {
		match *self {
			Either::Left(ref mut wrapped) => wrapped.next(),
			Either::Right(ref mut wrapped) => wrapped.next()
		}
	}
	
	fn is_exhausted(&self) -> bool {
		match *self {
			Either::Left(ref wrapped) => wrapped.is_exhausted(),
			Either::Right(ref wrapped) => wrapped.is_exhausted()
		}
	}
}

/// A silent source.
pub struct EquilibriumSource<F> {
	phantom: PhantomData<F>
}

impl<F> EquilibriumSource<F> {
	pub fn new() -> EquilibriumSource<F> { EquilibriumSource { phantom: PhantomData } }
}

impl<F> AudioSource for EquilibriumSource<F> where F: Frame {
	fn sample_hz(&self) -> f64 { 44_100.0 }
}

impl<F> Signal for EquilibriumSource<F> where F: Frame {
	type Frame = F;

	fn next(&mut self) -> F { F::equilibrium() }
}
