//! Defines a default output format that is used
//! internally by the DSP processing.

pub type Output = f32;
pub const CHANNELS: usize = 2;
pub type Frame = [Output; CHANNELS];

pub fn empty_frame() -> Frame { [0.0, 0.0] }
