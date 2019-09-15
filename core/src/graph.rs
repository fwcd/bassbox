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

/// Creates a new, shared atomically reference-counted
/// audio graph with a single, empty master node.
pub fn new_shared_graph() -> SharedAudioGraph {
	let mut graph = AudioGraph::new();
	let master = graph.add_node(DspNode::Empty);
	graph.set_master(Some(master));
	Arc::new(Mutex::new(graph))
}
