use jsonrpc_core::Result as RpcResult;
use jsonrpc_derive::rpc;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
/// A node in the audio graph.
pub enum RpcNode {
	/// An empty/passthrough node
	Empty,
	/// An actively silencing node
	Silence,
	/// A volume-controllable node
	#[serde(rename_all = "camelCase")]
	Volume {
		#[serde(default)] level: f32
	},
	/// A file source
	#[serde(rename_all = "camelCase")]
	File {
		file_path: String,
		#[serde(default)] paused: bool
	},
	/// A command source
	#[serde(rename_all = "camelCase")]
	Command {
		command: String,
		#[serde(default)] args: Vec<String>,
		sample_hz: f64,
		paused: bool
	},
	/// A dynamically dispatched source
	DynSource,
	/// A lowpass filter
	#[serde(rename_all = "camelCase")]
	IIRLowpass {
		#[serde(default)] cutoff_hz: f32,
		#[serde(default)] disabled: bool
	},
	/// A highpass filter
	#[serde(rename_all = "camelCase")]
	IIRHighpass {
		#[serde(default)] cutoff_hz: f32,
		#[serde(default)] disabled: bool
	},
	/// A dynamically dispatched filter (note that setting these is currently not supported)
	DynFilter,
	/// Any other node that currently has no RPC-serializable equivalent
	Other
}

pub type RpcNodeIndex = usize;
pub type RpcEdgeIndex = usize;

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
/// An edge in the audio graph.
pub struct RpcEdge {
	pub src: RpcNodeIndex,
	pub dest: RpcNodeIndex
}

impl RpcEdge {
	pub fn between(src: RpcNodeIndex, dest: RpcNodeIndex) -> RpcEdge {
		RpcEdge { src: src, dest: dest }
	}
}

#[derive(Serialize, Deserialize, Debug)]
/// The audio graph.
pub struct RpcGraph {
	pub nodes: HashMap<RpcNodeIndex, RpcNode>,
	pub edges: Vec<RpcEdge>,
	pub master: Option<RpcNodeIndex>
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

/// The audio graph service client generated by
/// jsonrpc_derive.
pub type AudioGraphServiceClient = gen_client::Client;
