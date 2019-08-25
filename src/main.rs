mod services;
mod sink;
mod dspnodes;

use services::player::{AudioPlayerServiceRpc, AudioPlayerService};
use jsonrpc_core::IoHandler;
use jsonrpc_stdio_server::ServerBuilder;

fn main() {
	let mut io = IoHandler::default();
	io.extend_with(AudioPlayerService::using_default_output()
		.expect("Could not fetch default output device")
		.to_delegate());
	
	ServerBuilder::new(io).build();
}
