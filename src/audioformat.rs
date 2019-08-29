//! Defines a default output format that is used
//! internally by the DSP processing.

use dsp::Frame;

pub type StandardOutput = f32;
pub const STANDARD_CHANNELS: usize = 2;
pub type StandardFrame = [StandardOutput; STANDARD_CHANNELS];

pub fn empty_standard_frame() -> StandardFrame { StandardFrame::equilibrium() }
