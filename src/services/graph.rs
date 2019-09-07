use jsonrpc_core::Result as RpcResult;
use jsonrpc_core::{Error as RpcError, ErrorCode as RpcErrorCode};
use jsonrpc_derive::rpc;
use serde::{Serialize, Deserialize};
use crate::processing::{DspNode, filter::{Disableable, CutoffFreq, IIRHighpassFilter, IIRLowpassFilter}};
use crate::graph::{AudioGraph, SharedAudioGraph, NodeIndex, EdgeIndex};
use crate::engine::BackgroundEngine;

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

impl From<&DspNode> for RpcNode {
	fn from(node: &DspNode) -> RpcNode {
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
}

impl RpcNode {
	pub fn into_dsp_node(self, sample_hz: f64) -> RpcResult<DspNode> {
		match self {
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

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct RpcNodeIndex {
	pub index: usize
}

impl From<NodeIndex> for RpcNodeIndex {
	fn from(index: NodeIndex) -> RpcNodeIndex { RpcNodeIndex { index: index.index() } }
}

impl Into<NodeIndex> for RpcNodeIndex {
	fn into(self) -> NodeIndex { NodeIndex::new(self.index) }
}

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

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct RpcEdgeIndex {
	pub index: usize
}

impl From<EdgeIndex> for RpcEdgeIndex {
	fn from(index: EdgeIndex) -> RpcEdgeIndex { RpcEdgeIndex { index: index.index() } }
}

impl Into<EdgeIndex> for RpcEdgeIndex {
	fn into(self) -> EdgeIndex { EdgeIndex::new(self.index) }
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
			edges: graph.edge_iter().map(|edge| RpcEdge::between(RpcNodeIndex::from(edge.src), RpcNodeIndex::from(edge.dest))).collect()
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
	
	/// Removes a node from the graph
	#[rpc(name = "audioGraph.removeNode")]
	fn remove_node(&self, node: RpcNodeIndex) -> RpcResult<()>;
	
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
		node.into_dsp_node(self.engine.sample_hz).map(|node| RpcNodeIndex::from(self.shared_graph.lock().unwrap().add_node(node)))
	}
	
	fn remove_node(&self, node: RpcNodeIndex) -> RpcResult<()> {
		self.shared_graph.lock().unwrap().remove_node(node.into());
		Ok(())
	}
	
	fn add_edge(&self, edge: RpcEdge) -> RpcResult<RpcEdgeIndex> {
		match self.shared_graph.lock().unwrap().add_edge(edge.src.into(), edge.dest.into()) {
			Ok(edge_index) => Ok(RpcEdgeIndex::from(edge_index)),
			Err(..) => Err(RpcError {
				code: RpcErrorCode::InvalidParams,
				message: "This edge would create a cycle in the graph".to_owned(),
				data: None
			})
		}
	}
}
