use dsp::Graph;
use std::sync::{Arc, Mutex};
use crate::processing::DspNode;
use crate::audioformat::StandardFrame;

/// The audio graph which is shared between
/// an engine and possibly control threads
/// (such as RPC-mechanisms).
#[derive(Clone)]
pub struct SharedAudioGraph {
	pub graph: Arc<Mutex<Graph<StandardFrame, DspNode>>>
}

impl SharedAudioGraph {
	pub fn new() -> SharedAudioGraph {
		SharedAudioGraph {
			graph: Arc::new(Mutex::new(Graph::new()))
		}
	}
}
