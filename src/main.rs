mod engine;
mod services;
mod sink;
mod processing;
mod context;
mod audioformat;

use std::thread;
use context::AudioContext;
use engine::{AudioEngine, cpal::CpalEngine};
use services::player::{AudioPlayerServiceRpc, AudioPlayerService};
use jsonrpc_core::IoHandler;
use jsonrpc_stdio_server::ServerBuilder;

fn main() {
	let context = AudioContext::new();
	run_engine_in_background(context);

	// Setup RPC
	let mut io = IoHandler::default();
	io.extend_with(AudioPlayerService::using_default_output()
		.expect("Could not fetch default output device")
		.to_delegate());
	
	ServerBuilder::new(io).build();
}

fn run_engine_in_background(context: AudioContext) {
	thread::spawn(move || {
		CpalEngine.run(context);
	});
}
