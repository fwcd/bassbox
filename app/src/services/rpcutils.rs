use jsonrpc_core::error::{Error, ErrorCode};

pub fn server_error(message: impl Into<String>) -> Error {
	Error {
		code: ErrorCode::ServerError(-32000),
		message: message.into(),
		data: None
	}
}
