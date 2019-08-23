use jsonrpc_core::Result as RpcResult;
use jsonrpc_derive::rpc;
use std::fs::File;
use std::path::Path;
use std::io::BufReader;
use rodio::Sink;

/// The audio playing service methods exposed via JSON-RPC.
#[rpc]
pub trait AudioPlayerServiceRpc {
	#[rpc(name = "audioPlayer.play")]
	fn play(&self);
	
	#[rpc(name = "audioPlayer.pause")]
	fn pause(&self);
}

pub struct AudioPlayerService {
	sink: Sink
}

impl AudioPlayerService {
	/// Creates a new player service that immediately
	/// begins playing the song referenced by the file path.
	pub fn from_file(path: impl AsRef<Path>) -> Option<AudioPlayerService> {
		let device = rodio::default_output_device()?;
		let file = File::open(path).ok()?;
		let source = rodio::Decoder::new(BufReader::new(file)).ok()?;
		let sink = Sink::new(&device);
		sink.append(source);
		Some(AudioPlayerService { sink: sink })
	}
}

impl AudioPlayerServiceRpc for AudioPlayerService {
	/// Continues playback after a pause.
	fn play(&self) {
		self.sink.play();
	}
	
	/// Pauses playback.
	fn pause(&self) {
		self.sink.pause();
	}
}
