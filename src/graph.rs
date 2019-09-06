use std::sync::{Arc, Mutex};
use dsp::Node;
use crate::processing::DspNode;
use crate::audioformat::StandardFrame;

pub type NodeIndex = dsp::NodeIndex;
pub type EdgeIndex = dsp::EdgeIndex;
pub type WouldCycle = dsp::WouldCycle;

/// A wrapper around the dsp graph whose node
/// indices remain stable after removals.
pub struct AudioGraph {
	inner: dsp::Graph<StandardFrame, DspNode>,
	free: Vec<bool>
}

impl AudioGraph {
	pub fn new() -> AudioGraph {
		AudioGraph { inner: dsp::Graph::new(), free: Vec::new() }
	}

	/// Adds the given node to the graph in O(1)
	pub fn add_node(&mut self, node: DspNode) -> NodeIndex {
		let index = self.inner.add_node(node);
		self.free.insert(index.index(), false);
		index
	}
	
	/// Adds the given edge to the graph
	pub fn add_edge(&mut self, src: NodeIndex, dest: NodeIndex) -> Result<EdgeIndex, WouldCycle> {
		self.inner.add_connection(src, dest)
	}

	/// Adds the given new node and input edge to the graph	
	pub fn add_input(&mut self, src: DspNode, dest: NodeIndex) -> (EdgeIndex, NodeIndex) {
		let (edge_index, node_index) = self.inner.add_input(src, dest);
		self.free.insert(node_index.index(), false);
		(edge_index, node_index)
	}
	
	/// Checks whether the node at the given index exists
	fn node_exists(&self, node: NodeIndex) -> bool {
		let i = node.index();
		if i < 0 || i > self.free.len() {
			false
		} else {
			!self.free[i]
		}
	}
	
	/// An immutable reference to the node at the given index
	pub fn node(&self, node: NodeIndex) -> Option<&DspNode> {
		if self.node_exists(node) {
			self.inner.node(node)
		} else {
			None
		}
	}
	
	/// A mutable reference to the node at the given index
	pub fn node_mut(&mut self, node: NodeIndex) -> Option<&mut DspNode> {
		if self.node_exists(node) {
			self.inner.node_mut(node)
		} else {
			None
		}
	}
	
	/// An iterator over all (possibly freed) nodes at their corresponding indices
	pub fn node_iter(&self) -> NodeIterator {
		NodeIterator { nodes: self.inner.raw_nodes(), free: &self.free, i: 0 }
	}
	
	/// An iterator over all edges in the graph
	pub fn edge_iter(&self) -> EdgeIterator {
		EdgeIterator { edges: self.inner.raw_edges(), i: 0 }
	}
	
	/// Sets the master (output) node of this graph
	pub fn set_master(&mut self, master: Option<NodeIndex>) {
		self.inner.set_master(master);
	}
	
	/// Requests audio from the master node
	#[inline]
	pub fn audio_requested(&mut self, output: &mut [StandardFrame], sample_hz: f64) {
		self.inner.audio_requested(output, sample_hz)
	}
}

/// An iterator over the nodes in an AudioGraph
pub struct NodeIterator<'a> {
	nodes: dsp::RawNodes<'a, DspNode>,
	free: &'a Vec<bool>,
	i: usize
}

impl<'a> Iterator for NodeIterator<'a> {
	type Item = Option<&'a DspNode>;
	
	fn next(&mut self) -> Option<Option<&'a DspNode>> {
		if self.i < self.nodes.len() {
			let node = Some(&self.nodes[self.i].weight).filter(|_| !self.free[self.i]);
			self.i += 1;
			Some(node)
		} else {
			None
		}
	}
}

/// A directed edge
pub struct Edge {
	pub src: NodeIndex,
	pub dest: NodeIndex
}

/// An iterator over the edges in an AudioGraph
pub struct EdgeIterator<'a> {
	edges: dsp::RawEdges<'a, StandardFrame>,
	i: usize
}

impl<'a> Iterator for EdgeIterator<'a> {
	type Item = Edge;
	
	fn next(&mut self) -> Option<Edge> {
		if self.i < self.edges.len() {
			let raw_edge = &self.edges[self.i];
			self.i += 1;
			Some(Edge { src: raw_edge.source(), dest: raw_edge.target() })
		} else {
			None
		}
	}
}

/// The audio graph which is shared between
/// an engine and possibly control threads
/// (such as RPC-mechanisms).
pub type SharedAudioGraph = Arc<Mutex<AudioGraph>>;

pub fn new_shared_graph() -> SharedAudioGraph {
	Arc::new(Mutex::new(AudioGraph::new()))
}
