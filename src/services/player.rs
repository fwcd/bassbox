use jsonrpc_core::Result as RpcResult;
use jsonrpc_derive::rpc;
use super::rpcutils::server_error;
use std::fs::File;
use std::io::BufReader;
use dsp::NodeIndex;
use dsp::sample::rate::Converter;
use crate::source::{AudioSource, mp3::Mp3Source};
use crate::graph::SharedAudioGraph;
use crate::processing::{DspNode, PauseState};
use crate::engine::{BackgroundEngine, ControlMsg};

/// The audio playing service methods exposed via JSON-RPC.
#[rpc]
pub trait AudioPlayerServiceRpc {
	/// Adds a song file to the queue.
	#[rpc(name = "audioPlayer.enqueueFile")]
	fn enqueue_file(&self, path: String) -> RpcResult<()>;
	
	/// Continues playback after a pause.
	#[rpc(name = "audioPlayer.play")]
	fn play(&self) -> RpcResult<()>;
	
	/// Pauses playback.
	#[rpc(name = "audioPlayer.pause")]
	fn pause(&self) -> RpcResult<()>;
	
	/// Controls volume
	#[rpc(name = "audioPlayer.setVolume")]
	fn set_volume(&self, volume: f32) -> RpcResult<()>;
}

/// A high-level audio playing facility that
/// abstracts the audio graph by providing convenient
/// queueing operations.
/// 
/// TODO: Provide a lower-level RPC service that allows
///       direct manipulation of the audio graph.
pub struct AudioPlayerService {
	shared_graph: SharedAudioGraph,
	src_node: NodeIndex,
	volume_node: NodeIndex,
	engine: BackgroundEngine
}

impl AudioPlayerService {
	/// Creates a new audio service which initializes
	/// the provided shared graph. In a sense, the
	/// audio service takes "primary ownership" over
	/// the graph while other referrers (such as audio
	/// engines) only query its output.
	pub fn constructing_graph(shared_graph: SharedAudioGraph, engine: BackgroundEngine) -> AudioPlayerService {
		let src_node: NodeIndex;
		let volume_node: NodeIndex;
		{
			// Initialize audio graph
			let mut graph = shared_graph.lock().unwrap();

			let master_node = graph.add_node(DspNode::Empty);
			volume_node = graph.add_input(DspNode::Empty, master_node).1;
			src_node = graph.add_input(DspNode::Empty, volume_node).1;

			graph.set_master(Some(master_node));
		}
		AudioPlayerService {
			shared_graph: shared_graph,
			src_node: src_node,
			volume_node: volume_node,
			engine: engine,
		}
	}
}

impl AudioPlayerServiceRpc for AudioPlayerService {
	fn enqueue_file(&self, path: String) -> RpcResult<()> {
		let file = File::open(&path)
			.map_err(|e| server_error(format!("Could not find file at '{}': {}", &path, e)))?;
		let source = Mp3Source::new(BufReader::new(file)); // TODO: Handle file type errors
		let source_hz = source.sample_hz();

		let mut graph = self.shared_graph.lock().unwrap();
		let src_ref = graph.node_mut(self.src_node).expect("Audio graph has no source node");
		
		// TODO: Queueing?
		*src_ref = DspNode::Source {
			src: Converter::from_hz_to_hz(Box::new(source), source_hz, self.engine.sample_hz),
			state: PauseState::Playing
		};
		
		Ok(())
	}
	
	fn play(&self) -> RpcResult<()> {
		self.engine.controls.send(ControlMsg::Play);
		Ok(())
	}
	
	fn pause(&self) -> RpcResult<()> {
		self.engine.controls.send(ControlMsg::Pause);
		Ok(())
	}
	
	fn set_volume(&self, volume: f32) -> RpcResult<()> {
		let mut graph = self.shared_graph.lock().unwrap();
		let volume_ref = graph.node_mut(self.volume_node).expect("Audio graph has no volume node");
		*volume_ref = DspNode::Volume(volume);
		Ok(())
	}
}
