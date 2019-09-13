use std::process::{ChildStdout, Command, Stdio};
use byteorder::{BigEndian, ReadBytesExt};
use crate::audioformat::StandardFrame;
use super::AudioSource;

/// An audio source that reads 32-bit floating
/// point PCM samples in big-endian from a
/// subprocess' stdout.
pub struct CommandSource {
	process: ChildStdout,
	command: String,
	args: Vec<String>,
	source_sample_hz: f64
}

impl CommandSource {
	pub fn new(command: &str, args: &[&str], source_sample_hz: f64) -> Result<CommandSource, String> {
		Ok(CommandSource {
			process: Command::new(command)
				.args(args)
				.stdin(Stdio::null())
				.stderr(Stdio::null())
				.stdout(Stdio::piped())
				.spawn().map_err(|e| format!("{:?}", e).to_owned())?
				.stdout.map_or_else(|| Err("Could not fetch command's stdout".to_owned()), Ok)?,
			command: command.to_owned(),
			args: args.iter().map(|s| (*s).to_owned()).collect(),
			source_sample_hz: source_sample_hz
		})
	}
	
	fn read_sample(&mut self) -> Option<f32> {
		self.process.read_f32::<BigEndian>().ok()
	}
	
	pub fn command(&self) -> &str { self.command.as_ref() }
	
	pub fn args(&self) -> &[String] { &self.args }
}

impl AudioSource for CommandSource {
	fn sample_hz(&self) -> f64 { self.source_sample_hz }
}

impl Iterator for CommandSource {
	type Item = StandardFrame;
	
	fn next(&mut self) -> Option<StandardFrame> {
		let left = self.read_sample()?;
		let right = self.read_sample()?;
		Some([left, right])
	}
}
