use jsonrpc_core::Result as RpcResult;
use jsonrpc_core::{Error as RpcError, ErrorCode as RpcErrorCode};
use dsp::EdgeRef;
use bassbox_rpc_api::{RpcNode, RpcNodeIndex, RpcEdge, RpcEdgeIndex, RpcGraph, AudioGraphServiceRpc};
use super::rpcutils::server_error;
use bassbox_core::processing::{DspNode, filter::{Disableable, CutoffFreq, IIRHighpassFilter, IIRLowpassFilter}};
use bassbox_core::graph::{AudioGraph, SharedAudioGraph};
use bassbox_core::source::{AudioSource, pausable::Pausable, conv::Converting, file::FileSource, command::CommandSource};
use bassbox_core::engine::BackgroundEngine;

/// The audio graph service implementation that holds a
/// reference to the shared audio graph and the engine's
/// control channel.
pub struct AudioGraphService {
	shared_graph: SharedAudioGraph<DspNode>,
	engine: BackgroundEngine
}

impl AudioGraphService {
	pub fn using_graph(shared_graph: SharedAudioGraph<DspNode>, engine: BackgroundEngine) -> AudioGraphService {
		AudioGraphService { shared_graph: shared_graph, engine: engine }
	}
}

impl AudioGraphServiceRpc for AudioGraphService {
	fn get(&self) -> RpcResult<RpcGraph> {
		let graph = self.shared_graph.lock().unwrap();
		Ok(RpcGraph::from_audio_graph(&graph))
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
		match self.shared_graph.lock().unwrap().add_connection(edge.src.into(), edge.dest.into()) {
			Ok(edge_index) => Ok(edge_index.index()),
			Err(..) => Err(RpcError {
				code: RpcErrorCode::InvalidParams,
				message: "This edge would create a cycle in the graph".to_owned(),
				data: None
			})
		}
	}
}

trait FromDspNodeExt {
	fn from_dsp_node(node: &DspNode) -> Self;
}

impl FromDspNodeExt for RpcNode {
	fn from_dsp_node(node: &DspNode) -> RpcNode {
		match *node {
			DspNode::Empty => RpcNode::Empty,
			DspNode::Silence => RpcNode::Silence,
			DspNode::File(Pausable { wrapped: ref converting_source, paused }) => RpcNode::File {
				file_path: converting_source.wrapped().file_path().to_owned(),
				paused: paused
			},
			DspNode::Command(Pausable { wrapped: ref converting_source, paused }) => RpcNode::Command {
				command: converting_source.wrapped().command().to_owned(),
				args: converting_source.wrapped().args().iter().map(|s| s.to_owned()).collect(),
				sample_hz: converting_source.wrapped().sample_hz(),
				paused: paused
			},
			DspNode::DynSource(..) => RpcNode::DynSource,
			DspNode::Volume(volume) => RpcNode::Volume { level: volume },
			DspNode::IIRLowpass(Disableable { wrapped: ref filter, disabled }) => RpcNode::IIRLowpass { cutoff_hz: filter.cutoff_hz(), disabled: disabled },
			DspNode::IIRHighpass(Disableable { wrapped: ref filter, disabled }) => RpcNode::IIRHighpass { cutoff_hz: filter.cutoff_hz(), disabled: disabled },
			DspNode::DynFilter(..) => RpcNode::DynFilter,
			_ => RpcNode::Other
		}
	}
}

trait IntoDspNodeExt {
	fn into_dsp_node(self, target_sample_hz: f64) -> RpcResult<DspNode>;
}

impl IntoDspNodeExt for RpcNode {
	fn into_dsp_node(self, target_sample_hz: f64) -> RpcResult<DspNode> {
		match self {
			RpcNode::Empty => Ok(DspNode::Empty),
			RpcNode::Silence => Ok(DspNode::Silence),
			RpcNode::Volume { level } => Ok(DspNode::Volume(level)),
			RpcNode::File { ref file_path, paused } => Ok(DspNode::File(
				Pausable::new(
					Converting::to_sample_hz(
						target_sample_hz,
						FileSource::new(file_path.as_ref()).map_err(|e| server_error(e))?
					),
					paused
				)
			)),
			RpcNode::Command { ref command, ref args, sample_hz, paused } => Ok(DspNode::Command(
				Pausable::new(
					Converting::to_sample_hz(
						target_sample_hz,
						CommandSource::new(command, &args.iter().map(|s| s.as_ref()).collect::<Vec<_>>(), sample_hz).map_err(|e| server_error(e))?
					),
					paused
				)
			)),
			RpcNode::IIRLowpass { cutoff_hz, disabled } => Ok(DspNode::IIRLowpass(Disableable::new(IIRLowpassFilter::from_cutoff_hz(cutoff_hz, target_sample_hz), disabled))),
			RpcNode::IIRHighpass { cutoff_hz, disabled } => Ok(DspNode::IIRHighpass(Disableable::new(IIRHighpassFilter::from_cutoff_hz(cutoff_hz, target_sample_hz), disabled))),
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

trait FromAudioGraphExt {
	fn from_audio_graph(graph: &AudioGraph<DspNode>) -> Self;
}

impl FromAudioGraphExt for RpcGraph {
	fn from_audio_graph(graph: &AudioGraph<DspNode>) -> RpcGraph {
		RpcGraph {
			nodes: graph.node_references().map(|(id, node)| (id.index(), RpcNode::from_dsp_node(node))).collect(),
			edges: graph.edge_references().map(|edge| RpcEdge::between(edge.source().index(), edge.target().index())).collect(),
			master: graph.master_index().map(|i| i.index())
		}
	}
}
