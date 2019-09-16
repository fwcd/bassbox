use dsp::{Signal, Frame};
use super::AudioSource;

/// A source that can be paused. When paused, the
/// source will not request any frames from its
/// wrapped source.
pub struct Pausable<S> {
	pub wrapped: S,
	pub paused: bool
}

impl<S> Pausable<S> {
	pub fn new(wrapped: S, paused: bool) -> Pausable<S> { Pausable { wrapped: wrapped, paused: paused } }

	pub fn paused(wrapped: S) -> Pausable<S> { Pausable { wrapped: wrapped, paused: true } }
	
	pub fn playing(wrapped: S) -> Pausable<S> { Pausable { wrapped: wrapped, paused: false } }
	
	pub fn with<T>(&self, wrapped: T) -> Pausable<T> { Pausable { wrapped: wrapped, paused: self.paused } }
}

impl<S> AudioSource for Pausable<S> where S: AudioSource {
	fn sample_hz(&self) -> f64 { self.wrapped.sample_hz() }
}

impl<S> Signal for Pausable<S> where S: Signal {
	type Frame = S::Frame;
	
	fn next(&mut self) -> S::Frame {
		if self.paused {
			S::Frame::equilibrium()
		} else {
			self.wrapped.next()
		}
	}
	
	fn is_exhausted(&self) -> bool { self.wrapped.is_exhausted() }
}
