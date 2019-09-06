use jsonrpc_core::Result as RpcResult;
use jsonrpc_core::{Error as RpcError, ErrorCode as RpcErrorCode};
use jsonrpc_derive::rpc;
use serde::{Serialize, Deserialize};
use crate::audioformat::StandardFrame;
use crate::processing::{DspNode, filter::{Disableable, CutoffFreq, IIRHighpassFilter, IIRLowpassFilter}};
use crate::graph::{AudioGraph, SharedAudioGraph};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RpcNode {
	Empty,
	Silence,
	DynSource,
	Volume(f32),
	IIRLowpass { cutoff_hz: f32, disabled: bool }, // TODO: Store cutoff_hz by value here
	IIRHighpass { cutoff_hz: f32, disabled: bool }, // TODO: Store cutoff_hz by value here
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
			DspNode::IIRLowpass(Disableable { wrapped: ref filter, disabled }) => RpcNode::IIRLowpass { cutoff_hz: filter.cutoff_hz(), disabled: disabled },
			DspNode::IIRHighpass(Disableable { wrapped: ref filter, disabled }) => RpcNode::IIRHighpass { cutoff_hz: filter.cutoff_hz(), disabled: disabled },
			DspNode::DynFilter(..) => RpcNode::DynFilter,
			_ => RpcNode::Other
		}
	}
	
	pub fn to_dsp_node(&self, sample_hz: f64) -> RpcResult<DspNode> {
		match *self {
			RpcNode::Empty => Ok(DspNode::Empty),
			RpcNode::Silence => Ok(DspNode::Silence),
			RpcNode::Volume(volume) => Ok(DspNode::Volume(volume)),
			RpcNode::IIRLowpass { cutoff_hz, disabled } => Ok(DspNode::IIRLowpass(Disableable { wrapped: IIRLowpassFilter::from_cutoff_hz(cutoff_hz, sample_hz), disabled: disabled })),
			RpcNode::IIRHighpass { cutoff_hz, disabled } => Ok(DspNode::IIRHighpass(Disableable { wrapped: IIRHighpassFilter::from_cutoff_hz(cutoff_hz, sample_hz), disabled: disabled })),
			RpcNode::DynFilter | RpcNode::DynSource => Err(RpcError {
				code: RpcErrorCode::InvalidParams,
				message: "Dynamic DSP nodes can currently not be crated from RPC nodes".to_owned(),
				data: None
			}),
			RpcNode::Other => Err(RpcError {
				code: RpcErrorCode::InvalidParams,
				message: "'Other' RPC node can not be converted to a DSP node".to_owned(),
				data: None
			})
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
	pub nodes: Vec<Option<RpcNode>>,
	pub edges: Vec<RpcEdge>
}

impl RpcGraph {
	pub fn from(graph: &AudioGraph) -> RpcGraph {
		RpcGraph {
			nodes: graph.node_iter().map(|opt_node| opt_node.map(|node| RpcNode::from(node))).collect(),
			edges: graph.edge_iter().map(|edge| RpcEdge::between(edge.src.index(), edge.dest.index())).collect()
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
