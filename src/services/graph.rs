use jsonrpc_core::Result as RpcResult;
use jsonrpc_core::{Error as RpcError, ErrorCode as RpcErrorCode};
use jsonrpc_derive::rpc;
use serde::{Serialize, Deserialize};
use super::rpcutils::server_error;
use crate::processing::{DspNode, filter::{Disableable, CutoffFreq, IIRHighpassFilter, IIRLowpassFilter}};
use crate::graph::{AudioGraph, SharedAudioGraph};
use crate::source::{pausable::Pausable, conv::Converting, file::FileSource};
use crate::engine::BackgroundEngine;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RpcNode {
	Empty,
	Silence,
	Volume(f32),
	#[serde(rename_all = "camelCase")]
	File { file_path: String, #[serde(default)] paused: bool },
	#[serde(rename_all = "camelCase")]
	IIRLowpass { #[serde(default)] cutoff_hz: f32, #[serde(default)] disabled: bool },
	#[serde(rename_all = "camelCase")]
	IIRHighpass { #[serde(default)] cutoff_hz: f32, #[serde(default)] disabled: bool },
	DynFilter,
	Other
}

impl From<&DspNode> for RpcNode {
	fn from(node: &DspNode) -> RpcNode {
		match *node {
			DspNode::Empty => RpcNode::Empty,
			DspNode::Silence => RpcNode::Silence,
			DspNode::File(Pausable { wrapped: ref converting_source, paused }) => RpcNode::File { file_path: converting_source.wrapped().file_path().to_owned(), paused: paused },
			DspNode::Volume(volume) => RpcNode::Volume(volume),
			DspNode::IIRLowpass(Disableable { wrapped: ref filter, disabled }) => RpcNode::IIRLowpass { cutoff_hz: filter.cutoff_hz(), disabled: disabled },
			DspNode::IIRHighpass(Disableable { wrapped: ref filter, disabled }) => RpcNode::IIRHighpass { cutoff_hz: filter.cutoff_hz(), disabled: disabled },
			DspNode::DynFilter(..) => RpcNode::DynFilter,
			_ => RpcNode::Other
		}
	}
}

impl RpcNode {
	pub fn into_dsp_node(self, sample_hz: f64) -> RpcResult<DspNode> {
		match self {
			RpcNode::Empty => Ok(DspNode::Empty),
			RpcNode::Silence => Ok(DspNode::Silence),
			RpcNode::Volume(volume) => Ok(DspNode::Volume(volume)),
			RpcNode::File { ref file_path, paused } => Ok(DspNode::File(
				Pausable::new(
					Converting::to_sample_hz(
						sample_hz,
						FileSource::new(file_path.as_ref()).map_or_else(|| Err(server_error("Could not create file source")), Ok)?
					),
					paused
				)
			)),
			RpcNode::IIRLowpass { cutoff_hz, disabled } => Ok(DspNode::IIRLowpass(Disableable::new(IIRLowpassFilter::from_cutoff_hz(cutoff_hz, sample_hz), disabled))),
			RpcNode::IIRHighpass { cutoff_hz, disabled } => Ok(DspNode::IIRHighpass(Disableable::new(IIRHighpassFilter::from_cutoff_hz(cutoff_hz, sample_hz), disabled))),
			RpcNode::DynFilter => Err(RpcError {
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

type RpcNodeIndex = usize;
type RpcEdgeIndex = usize;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct RpcEdge {
	pub src: RpcNodeIndex,
	pub dest: RpcNodeIndex
}

impl RpcEdge {
	pub fn between(src: RpcNodeIndex, dest: RpcNodeIndex) -> RpcEdge {
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
	
	/// Adds a node to the graph, returning its index
	#[rpc(name = "audioGraph.addNode")]
	fn add_node(&self, node: RpcNode) -> RpcResult<RpcNodeIndex>;
	
	/// Creates a new template 
	
	/// Removes a node from the graph
	#[rpc(name = "audioGraph.removeNode")]
	fn remove_node(&self, index: RpcNodeIndex) -> RpcResult<()>;
	
	/// Replaces a node from the graph
	#[rpc(name = "audioGraph.replaceNode")]
	fn replace_node(&self, index: RpcNodeIndex, node: RpcNode) -> RpcResult<()>;
	
	/// Adds an edge to the graph
	#[rpc(name = "audioGraph.addEdge")]
	fn add_edge(&self, edge: RpcEdge) -> RpcResult<RpcEdgeIndex>;
	
	// TODO: removeEdge
}

pub struct AudioGraphService {
	shared_graph: SharedAudioGraph,
	engine: BackgroundEngine
}

impl AudioGraphService {
	pub fn using_graph(shared_graph: SharedAudioGraph, engine: BackgroundEngine) -> AudioGraphService {
		AudioGraphService { shared_graph: shared_graph, engine: engine }
	}
}

impl AudioGraphServiceRpc for AudioGraphService {
	fn get(&self) -> RpcResult<RpcGraph> {
		let graph = self.shared_graph.lock().unwrap();
		Ok(RpcGraph::from(&graph))
	}
	
	fn add_node(&self, node: RpcNode) -> RpcResult<RpcNodeIndex> {
		node.into_dsp_node(self.engine.sample_hz).map(|node| self.shared_graph.lock().unwrap().add_node(node).index())
	}
	
	fn remove_node(&self, index: RpcNodeIndex) -> RpcResult<()> {
		self.shared_graph.lock().unwrap().remove_node(index.into());
		Ok(())
	}
	
	fn replace_node(&self, index: RpcNodeIndex, node: RpcNode) -> RpcResult<()> {
		node.into_dsp_node(self.engine.sample_hz).and_then(|node| {
			let mut graph = self.shared_graph.lock().unwrap();
			let node_ref = graph.node_mut(index.into()).map_or_else(|| Err(server_error(format!("Node at {} does not exist", index))), Ok)?;
			*node_ref = node;
			Ok(())
		})
	}
	
	fn add_edge(&self, edge: RpcEdge) -> RpcResult<RpcEdgeIndex> {
		match self.shared_graph.lock().unwrap().add_edge(edge.src.into(), edge.dest.into()) {
			Ok(edge_index) => Ok(edge_index.index()),
			Err(..) => Err(RpcError {
				code: RpcErrorCode::InvalidParams,
				message: "This edge would create a cycle in the graph".to_owned(),
				data: None
			})
		}
	}
}
