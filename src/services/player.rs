use jsonrpc_core::Result as RpcResult;
use jsonrpc_derive::rpc;
use super::rpcutils::server_error;
use std::fs::File;
use std::io::BufReader;
use rodio::Sink;

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
}

pub struct AudioPlayerService {
	sink: Sink
}

impl AudioPlayerService {
	/// Creates a new player service using the default
	/// audio output device.
	pub fn using_default_output() -> Option<AudioPlayerService> {
		let device = rodio::default_output_device()?;
		Some(AudioPlayerService { sink: Sink::new(&device) })
	}
}

impl AudioPlayerServiceRpc for AudioPlayerService {
	fn enqueue_file(&self, path: String) -> RpcResult<()> {
		let file = File::open(&path)
			.map_err(|e| server_error(format!("Could not find file at '{}': {}", &path, e)))?;
		let source = rodio::Decoder::new(BufReader::new(file))
			.map_err(|e| server_error(format!("Could not decode file at '{}': {}", &path, e)))?;
		self.sink.append(source);
		Ok(())
	}
	
	fn play(&self) -> RpcResult<()> {
		self.sink.play();
		Ok(())
	}
	
	fn pause(&self) -> RpcResult<()> {
		self.sink.pause();
		Ok(())
	}
}
