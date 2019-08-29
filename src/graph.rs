use dsp::Graph;
use std::sync::{Arc, Mutex};
use crate::processing::DspNode;
use crate::audioformat::StandardFrame;

/// The audio graph which is shared between
/// an engine and possibly control threads
/// (such as RPC-mechanisms).
pub type SharedAudioGraph = Arc<Mutex<Graph<StandardFrame, DspNode>>>;

pub fn new_shared_graph() -> SharedAudioGraph {
	Arc::new(Mutex::new(Graph::new()))
}
