use crate::audioformat::StandardFrame;
use super::AudioSource;

/// A source that can be paused. When paused, the
/// source will not request any frames from its
/// wrapped source.
pub struct PausableSource<S> {
	pub wrapped: S,
	pub paused: bool
}

impl<S> PausableSource<S> {
	pub fn paused(wrapped: S) -> PausableSource<S> { PausableSource { wrapped: wrapped, paused: true } }
	
	pub fn playing(wrapped: S) -> PausableSource<S> { PausableSource { wrapped: wrapped, paused: true } }
	
	pub fn with<T>(&self, wrapped: T) -> PausableSource<T> { PausableSource { wrapped: wrapped, paused: self.paused } }
}

impl<S> AudioSource for PausableSource<S> where S: AudioSource {
	fn sample_hz(&self) -> f64 { self.wrapped.sample_hz() }
}

impl<S> Iterator for PausableSource<S> where S: AudioSource {
	type Item = StandardFrame;
	
	fn next(&mut self) -> Option<StandardFrame> {
		if self.paused {
			None
		} else {
			self.wrapped.next()
		}
	}
}
