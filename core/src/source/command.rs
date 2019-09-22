use std::process::{ChildStdout, ChildStdin, Command, Stdio};
use std::io::Write;
use byteorder::{BigEndian, ReadBytesExt};
use dsp::{Signal, Frame};
use crate::audioformat::StandardFrame;
use super::AudioSource;

/// An audio source that reads 32-bit floating
/// point PCM samples in big-endian from a
/// subprocess' stdout.
pub struct CommandSource {
	child_out: ChildStdout,
	child_in: Option<ChildStdin>,
	command: String,
	args: Vec<String>,
	source_sample_hz: f64,
	reached_end: bool,
}

// TODO: Make 'command' an intermediate/processing node

impl CommandSource {
	/// Creates a new CommandSource spawning the provided executable
	/// with the given args.
	/// 
	/// The `takes_input` flag optionally enables piped standard input, thus
	/// making it possible to feed audio to the child process.
	pub fn new(command: &str, args: &[&str], source_sample_hz: f64, takes_input: bool) -> Result<CommandSource, String> {
		let process = Command::new(command)
			.args(args)
			.stdin(if takes_input { Stdio::piped() } else { Stdio::null() })
			.stderr(Stdio::null())
			.stdout(Stdio::piped())      
			.spawn().map_err(|e| format!("{:?}", e))?;
		Ok(CommandSource {
			child_out: process.stdout.ok_or("Could not fetch child's stdout")?,
			child_in: if takes_input { Some(process.stdin.ok_or("Could not fetch child's stdin")?) } else { None },
			command: command.to_owned(),
			args: args.iter().map(|s| (*s).to_owned()).collect(),
			source_sample_hz: source_sample_hz,
			reached_end: false
		})
	}
	
	fn read_sample(&mut self) -> Option<f32> {
		self.child_out.read_f32::<BigEndian>().ok()
	}
	
	fn read_frame(&mut self) -> Option<StandardFrame> {
		let left = self.read_sample()?;
		let right = self.read_sample()?;
		Some([left, right])
	}
	
	/// Fetches the child processes stdin. Only present
	/// if `takes_input` was set.
	pub fn input(&self) -> Option<&impl Write> { self.child_in.as_ref() }
	
	/// Fetches the executable that was invoked.
	pub fn command(&self) -> &str { self.command.as_ref() }
	
	/// Fetches the arguments from the command.
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
