use dsp::Graph;
use std::sync::{Arc, Mutex};
use crate::processing::dspnode::DspNode;
use crate::audioformat::Frame;

/// The audio state which is shared between
/// an engine and possibly control threads
/// (such as RPC-mechanisms).
#[derive(Clone)]
pub struct AudioContext {
	pub graph: Arc<Mutex<Graph<Frame, DspNode>>>
}

impl AudioContext {
	pub fn new() -> AudioContext {
		AudioContext {
			graph: Arc::new(Mutex::new(Graph::new()))
		}
	}
}
