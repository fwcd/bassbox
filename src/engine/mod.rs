//! Contains blocking facilities that run
//! the audio graph and the sinks.

pub mod speaker;

use crate::graph::SharedAudioGraph;

/// A blocking audio playing engine.
pub trait AudioEngine {
	/// Runs the engine on a background thread.
	fn run_async(self, context: SharedAudioGraph) -> BackgroundEngine;
}

/// Represents an engine running asynchronously
/// in the background. Usually returned after
/// starting the engine.
pub struct BackgroundEngine {
	/// The output sample rate
	pub sample_hz: f64
}
