use std::process::{ChildStdout, Command, Stdio};
use byteorder::{BigEndian, ReadBytesExt};
use dsp::{Signal, Frame};
use crate::audioformat::StandardFrame;
use super::AudioSource;

/// An audio source that reads 32-bit floating
/// point PCM samples in big-endian from a
/// subprocess' stdout.
pub struct CommandSource {
	process: ChildStdout,
	command: String,
	args: Vec<String>,
	source_sample_hz: f64,
	reached_end: bool
}

// TODO: Make 'command' an intermediate/processing node

impl CommandSource {
	pub fn new(command: &str, args: &[&str], source_sample_hz: f64) -> Result<CommandSource, String> {
		Ok(CommandSource {
			process: Command::new(command)
				.args(args)
				.stdin(Stdio::null())
				.stderr(Stdio::null())
				.stdout(Stdio::piped())      
				.spawn().map_err(|e| format!("{:?}", e))?
				.stdout.map_or_else(|| Err("Could not fetch command's stdout".to_owned()), Ok)?,
			command: command.to_owned(),
			args: args.iter().map(|s| (*s).to_owned()).collect(),
			source_sample_hz: source_sample_hz,
			reached_end: false
		})
	}
	
	fn read_sample(&mut self) -> Option<f32> {
		self.process.read_f32::<BigEndian>().ok()
	}
	
	fn read_frame(&mut self) -> Option<StandardFrame> {
		let left = self.read_sample()?;
		let right = self.read_sample()?;
		Some([left, right])
	}
	
	pub fn command(&self) -> &str { self.command.as_ref() }
	
	pub fn args(&self) -> &[String] { &self.args }
}

impl AudioSource for CommandSource {
	fn sample_hz(&self) -> f64 { self.source_sample_hz }
}

impl Signal for CommandSource {
	type Frame = StandardFrame;
	
	fn next(&mut self) -> StandardFrame {
		if self.reached_end {
			StandardFrame::equilibrium()
		} else {
			self.read_frame().unwrap_or_else(|| {
				self.reached_end = true;
				StandardFrame::equilibrium()
			})
		}
	}
	
	fn is_exhausted(&self) -> bool { self.reached_end }
}
