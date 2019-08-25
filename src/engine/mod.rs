//! Contains blocking facilities that run
//! the audio graph and the sinks.

pub mod cpal;

use crate::context::AudioContext;

/// A blocking audio playing engine.
pub trait AudioEngine {
	/// Runs the engine and takes control over the thread.
	fn run(self, context: AudioContext);
}
