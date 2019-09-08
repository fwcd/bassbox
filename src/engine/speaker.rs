use super::{AudioEngine, BackgroundEngine, ControlMsg, EngineControls};
use crate::audioformat::{StandardFrame, empty_standard_frame, STANDARD_CHANNELS, StandardSample};
use crate::graph::SharedAudioGraph;
use std::sync::mpsc;
use std::thread;
use dsp::sample::{Sample, Frame, FromSample, conv::ToFrameSliceMut};
use cpal::{StreamData, UnknownTypeOutputBuffer, OutputBuffer};
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};

macro_rules! with_buffer_of {
	($data: expr, $body: expr) => {
		match $data {
			StreamData::Output { buffer: UnknownTypeOutputBuffer::U16(ref mut buffer) } => $body(buffer),
			StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(ref mut buffer) } => $body(buffer),
			StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(ref mut buffer) } => $body(buffer),
			_ => println!("Audio format was not recognized by CPAL!")
		}
	};
}

/// An engine that uses CPAL to provide
/// cross-platform audio output.
pub struct SpeakerEngine;

impl AudioEngine for SpeakerEngine {
	fn run_async(self, shared_graph: SharedAudioGraph) -> BackgroundEngine {
		// Setup CPAL
		let host = cpal::default_host();
		let event_loop = host.event_loop();
		let device = host.default_output_device().expect("No output device available.");
		let format = device.default_output_format().expect("No default format.");
		let sample_hz = format.sample_rate.0 as f64;
		let channels = format.channels as usize;
		let stream_id = event_loop.build_output_stream(&device, &format).expect("Could not build output stream");
		
		if channels != STANDARD_CHANNELS {
			// TODO
			panic!("CPAL output format has {} channels, but only exactly {} are supported currently.", channels, STANDARD_CHANNELS);
		}
		
		let (control_sender, control_receiver) = mpsc::sync_channel(1);
		event_loop.play_stream(stream_id.clone()).expect("Could not play stream.");

		thread::spawn(move || {
			let mut paused = false;

			event_loop.run(move |_id, result| {
				let mut data = result.expect("Error while streaming.");
				
				// Possibly receive a control operation message
				if let Ok(msg) = control_receiver.try_recv() {
					match msg {
						ControlMsg::Play => paused = false,
						ControlMsg::Pause => {
							with_buffer_of!(data, write_silence);
							paused = true
						}
						// _ => println!("Control message not recognized by the speaker/CPAL engine: {:?}", msg)
					}
				}
			
				if !paused {
					// Play the audio
					if let StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(ref mut buffer) } = data {
						// Our speaker format matches the internal format, thus we do
						// not need to allocate an extra vector
						let buf_slice: &mut [StandardFrame] = buffer.to_frame_slice_mut().unwrap();
						shared_graph.lock().unwrap().audio_requested(buf_slice, sample_hz);
					} else {
						// Read audio from graph into temporary buffer
						let sample_count = buffer_sample_count(&data).unwrap_or(0);
						let frame_count = sample_count / STANDARD_CHANNELS;
						let mut audio: Vec<StandardFrame> = vec![empty_standard_frame(); frame_count];
						shared_graph.lock().unwrap().audio_requested(&mut audio, sample_hz);
					
						with_buffer_of!(data, |buffer| write_audio(&audio, buffer));
					}
				}
			});
		});

		BackgroundEngine { sample_hz: sample_hz, controls: EngineControls::new(control_sender) }
	}
}

/// Fetches CPAL's stream data buffer sample count.
fn buffer_sample_count(data: &StreamData) -> Option<usize> {
	match *data {
		StreamData::Output { buffer: UnknownTypeOutputBuffer::U16(ref buffer) } => Some(buffer.len()),
		StreamData::Output { buffer: UnknownTypeOutputBuffer::I16(ref buffer) } => Some(buffer.len()),
		StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(ref buffer) } => Some(buffer.len()),
		_ => None
	}
}

/// Writes the audio to the speaker in a specified format.
fn write_audio<S>(audio: &[StandardFrame], buffer: &mut OutputBuffer<S>) where S: cpal::Sample + Sample + FromSample<StandardSample> {
	let buf_slice: &mut [[S; STANDARD_CHANNELS]] = buffer.to_frame_slice_mut().unwrap();
	for (i, frame) in buf_slice.iter_mut().enumerate() {
		*frame = audio[i].map(Sample::to_sample);
	}
}

/// "Zeroes out" the buffer.
fn write_silence<S>(buffer: &mut OutputBuffer<S>) where S: cpal::Sample + FromSample<u16> {
	for i in 0..buffer.len() {
		buffer[i] = 0.to_sample();
	}
}
