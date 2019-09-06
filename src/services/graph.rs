use jsonrpc_core::Result as RpcResult;
use jsonrpc_derive::rpc;
use serde::{Serialize, Deserialize};
use dsp::Graph;
use crate::audioformat::StandardFrame;
use crate::processing::{DspNode, filter::{Disableable, CutoffFreq}};
use crate::graph::SharedAudioGraph;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RpcNode {
	Empty,
	Silence,
	DynSource,
	Volume(f32),
	IIRLowpass { cutoff_hz: f32 }, // TODO: Store cutoff_hz by value here
	IIRHighpass { cutoff_hz: f32 }, // TODO: Store cutoff_hz by value here
	DynFilter,
	Other
}

impl RpcNode {
	pub fn from(node: &DspNode) -> RpcNode {
		match *node {
			DspNode::Empty => RpcNode::Empty,
			DspNode::Silence => RpcNode::Silence,
			DspNode::DynSource { .. } => RpcNode::DynSource,
			DspNode::Volume(volume) => RpcNode::Volume(volume),
			DspNode::IIRLowpass(Disableable { wrapped: ref filter, .. }) => RpcNode::IIRLowpass { cutoff_hz: filter.cutoff_hz() },
			DspNode::IIRHighpass(Disableable { wrapped: ref filter, .. }) => RpcNode::IIRHighpass { cutoff_hz: filter.cutoff_hz() },
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
	pub fn using_graph(shared_graph: SharedAudioGraph) -> AudioGraphService {
		AudioGraphService { shared_graph: shared_graph }
	}
}

impl AudioGraphServiceRpc for AudioGraphService {
	fn get(&self) -> RpcResult<RpcGraph> {
		let graph = self.shared_graph.lock().unwrap();
		Ok(RpcGraph::from(&graph))
	}
}
