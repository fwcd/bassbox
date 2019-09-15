use std::sync::{Arc, Mutex};
use crate::processing::DspNode;
use crate::audioformat::StandardFrame;

pub type NodeIndex = dsp::NodeIndex;
pub type EdgeIndex = dsp::EdgeIndex;
pub type AudioGraph = dsp::Graph<StandardFrame, DspNode>;

/// The audio graph which is shared between
/// an engine and possibly control threads
/// (such as RPC-mechanisms).
pub type SharedAudioGraph = Arc<Mutex<AudioGraph>>;

pub fn new_shared_graph() -> SharedAudioGraph {
	Arc::new(Mutex::new(AudioGraph::new()))
}
