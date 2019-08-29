//! Contains blocking facilities that run
//! the audio graph and the sinks.

pub mod speaker;

use crate::graph::SharedAudioGraph;

/// A blocking audio playing engine.
pub trait AudioEngine {
	/// Runs the engine and takes control over the thread.
	fn run(self, context: SharedAudioGraph);
}
