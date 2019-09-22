use dsp::Node;
use std::sync::Arc;
use parking_lot::ReentrantMutex;
use crate::audioformat::StandardFrame;
use crate::util::empty::Empty;

pub type NodeIndex = dsp::NodeIndex;
pub type EdgeIndex = dsp::EdgeIndex;
pub type AudioGraph<N> = dsp::Graph<StandardFrame, N>;

/// The audio graph which is shared between
/// an engine and possibly control threads
/// (such as RPC-mechanisms).
pub type SharedAudioGraph<N> = Arc<ReentrantMutex<AudioGraph<N>>>;

/// Creates a new, shared atomically reference-counted
/// audio graph with a single, empty master node.
pub fn new_shared_graph<N>() -> SharedAudioGraph<N> where N: Empty + Node<StandardFrame> {
	let mut graph = AudioGraph::new();
	let master = graph.add_node(N::empty());
	graph.set_master(Some(master));
	Arc::new(ReentrantMutex::new(graph))
}
