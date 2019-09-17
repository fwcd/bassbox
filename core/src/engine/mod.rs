//! Contains blocking facilities that run
//! the audio graph and the sinks.

pub mod speaker;

use std::sync::mpsc;
use dsp::Node;
use crate::graph::SharedAudioGraph;
use crate::audioformat::StandardFrame;

/// A blocking audio playing engine.
pub trait AudioEngine {
	/// Runs the engine on a background thread.
	fn run_async<N>(self, shared_graph: SharedAudioGraph<N>) -> BackgroundEngine where N: Node<StandardFrame> + Send + 'static;
}

/// Represents an engine running asynchronously
/// in the background. Usually returned after
/// starting the engine.
#[derive(Clone)]
pub struct BackgroundEngine {
	/// The output sample rate
	pub sample_hz: f64,
	pub controls: EngineControls
}

/// A single control operation which can be
/// sent to an engine through EngineControls.
#[derive(Debug)]
pub enum ControlMsg {
	Pause,
	Play,
	Custom(String)
}

/// A wrapper around an MPSC channel that
/// allows you to send a control (message)
/// to the engine.
/// 
/// Note that these _engine-level_ operations
/// are completely unrelated to _graph/source-level_
/// operations (such as `Pausable`).
#[derive(Clone)]
pub struct EngineControls {
	tx: mpsc::SyncSender<ControlMsg>
}

impl EngineControls {
	pub fn new(tx: mpsc::SyncSender<ControlMsg>) -> EngineControls {
		EngineControls { tx: tx }
	}
	
	/// Sends a control message to the engine.
	pub fn send(&self, msg: ControlMsg) {
		self.tx.send(msg).expect("Could not send control message")
	}
}
