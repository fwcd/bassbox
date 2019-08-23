mod services;

use services::player::{AudioPlayerServiceRpc, AudioPlayerService};
use jsonrpc_core::IoHandler;
use jsonrpc_stdio_server::ServerBuilder;

fn main() {
	let mut io = IoHandler::default();
	io.extend_with(AudioPlayerService::from_file("local/oldtownroad.mp3").unwrap().to_delegate());
	
	ServerBuilder::new(io).build();
}
