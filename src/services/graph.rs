use jsonrpc_core::Result as RpcResult;
use jsonrpc_derive::rpc;
use serde::{Serialize, Deserialize};
use dsp::Graph;
use crate::audioformat::StandardFrame;
use crate::processing::DspNode;
use crate::graph::SharedAudioGraph;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RpcNode {
	Empty,
	Silence,
	DynSource,
	Volume,
	IIRLowpass, // TODO: Store cutoff_hz by value here
	IIRHighpass, // TODO: Store cutoff_hz by value here
	DynFilter,
	Other
}

impl RpcNode {
	pub fn from(node: &DspNode) -> RpcNode {
		match *node {
			DspNode::Empty => RpcNode::Empty,
			DspNode::Silence => RpcNode::Silence,
			DspNode::DynSource { .. } => RpcNode::DynSource,
			DspNode::Volume(..) => RpcNode::Volume,
			DspNode::IIRLowpass { .. } => RpcNode::IIRLowpass,
			DspNode::IIRHighpass { .. } => RpcNode::IIRHighpass,
			DspNode::DynFilter(..) => RpcNode::DynFilter,
			_ => RpcNode::Other
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct RpcEdge {
	pub src: usize,
	pub dest: usize
}

impl RpcEdge {
	pub fn between(src: usize, dest: usize) -> RpcEdge {
		RpcEdge { src: src, dest: dest }
	}
}

#[derive(Serialize, Deserialize)]
pub struct RpcGraph {
	pub nodes: Vec<RpcNode>,
	pub edges: Vec<RpcEdge>
}

impl RpcGraph {
	pub fn from(graph: &Graph<StandardFrame, DspNode>) -> RpcGraph {
		RpcGraph {
			nodes: graph.raw_nodes().iter().map(|node| RpcNode::from(&node.weight)).collect(),
			edges: graph.raw_edges().iter().map(|edge| RpcEdge::between(edge.source().index(), edge.target().index())).collect()
		}
	}
}

/// The audio graph methods exposed via JSON-RPC
#[rpc]
pub trait AudioGraphServiceRpc {
	/// Fetches the current state of the graph
	#[rpc(name = "audioGraph.get")]
	fn get(&self) -> RpcResult<RpcGraph>;
}

pub struct AudioGraphService {
	shared_graph: SharedAudioGraph
}

impl AudioGraphService {
	pub fn with_graph(shared_graph: SharedAudioGraph) -> AudioGraphService {
		AudioGraphService { shared_graph: shared_graph }
	}
}

impl AudioGraphServiceRpc for AudioGraphService {
	fn get(&self) -> RpcResult<RpcGraph> {
		let graph = self.shared_graph.lock().unwrap();
		Ok(RpcGraph::from(&graph))
	}
}
