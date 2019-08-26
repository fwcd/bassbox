use super::Sink;

use crate::audioformat::Frame;
use std::sync::atomic::AtomicBool;

/// The engine's "native" sink which is implemented
/// in the engine directly. Can be enabled
/// or disabled.
pub struct DefaultSink {
	pub enabled: AtomicBool
}

impl DefaultSink {
	pub fn new() -> DefaultSink {
		DefaultSink { enabled: AtomicBool::new(true) }
	}
}

/// Marker implementation of Sink. Does
/// not do anything directly since the
/// relevant code is implemented in the
/// engine itself.
impl Sink for DefaultSink {
	fn output_audio(&self, audio: &[Frame]) {}
}
