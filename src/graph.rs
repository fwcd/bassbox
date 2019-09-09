use std::sync::{Arc, Mutex};
use dsp::Node;
use crate::processing::DspNode;
use crate::audioformat::StandardFrame;

pub type NodeIndex = dsp::NodeIndex;
pub type EdgeIndex = dsp::EdgeIndex;
pub type WouldCycle = dsp::WouldCycle;

/// A wrapper around the dsp graph whose node
/// indices remain stable after removals.
/// 
/// Note that _different_ nodes may get the _same_
/// index if a node is removed and another one added.
/// node indices should generally not outlive
/// their referenced node.
pub struct AudioGraph {
	inner: dsp::Graph<StandardFrame, DspNode>,
	free: Vec<bool>,
	has_free: bool
}

impl AudioGraph {
	pub fn new() -> AudioGraph {
		AudioGraph { inner: dsp::Graph::new(), free: Vec::new(), has_free: false }
	}

	/// Adds the given node to the graph in O(n)
	pub fn add_node(&mut self, node: DspNode) -> NodeIndex {
		if let Some(index) = self.take_free() {
			let node_ref = self.inner.node_mut(index).expect("A free node index should still hold a (no longer used) node");
			*node_ref = node;
			index
		} else {
			let index = self.inner.add_node(node);
			self.free.insert(index.index(), false);
			index
		}
	}
	
	/// Adds the given edge to the graph
	pub fn add_edge(&mut self, src: NodeIndex, dest: NodeIndex) -> Result<EdgeIndex, WouldCycle> {
		// TODO: Prevent duplicate edges?
		self.inner.add_connection(src, dest)
	}
	
	/// Adds the given new node and input edge to the graph	
	pub fn add_input(&mut self, src: DspNode, dest: NodeIndex) -> (EdgeIndex, NodeIndex) {
		let node_index = self.add_node(src);
		let edge_index = self.add_edge(node_index, dest).expect("Adding an input edge with a new node should never cause a cycle");
		(edge_index, node_index)
	}
	
	/// Takes the next free index (if available) in O(n)
	fn take_free(&mut self) -> Option<NodeIndex> {
		if self.has_free {
			let mut has_free = false;
			let mut taken: Option<NodeIndex> = None;

			for i in 0..self.node_count() {
				if self.free[i] {
					match taken {
						Some(..) => has_free = true,
						None => {
							self.free[i] = false;
							taken = Some(NodeIndex::new(i))
						}
					}
				}
			}

			self.has_free = has_free;
			taken
		} else {
			None
		}
	}
	
	/// Removes a node from the graph in O(1).
	/// 
	/// Any indices referring to the given node should
	/// be dropped by now.
	pub fn remove_node(&mut self, node: NodeIndex) {
		*self.node_mut(node).expect("Tried to remove non-existing node") = DspNode::Empty;
		self.free[node.index()] = true;
		self.has_free = true;
	}
	
	/// Checks whether the node at the given index exists
	fn node_exists(&self, node: NodeIndex) -> bool {
		let i = node.index();
		if i > self.free.len() {
			false
		} else {
			!self.free[i]
		}
	}
	
	/// The number of nodes in the graph
	pub fn node_count(&self) -> usize { self.inner.node_count() }
	
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
	
	/// Fetches the master (output) node of this graph
	pub fn master(&self) -> Option<NodeIndex> {
		self.inner.master_index()
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
