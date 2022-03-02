mod error;

use std::{fs::File, io::Cursor, path::Path, sync::Arc};

pub use error::*;

use symphonia::core::{
	audio::{AudioBuffer, AudioBufferRef, Signal},
	conv::{FromSample, IntoSample},
	io::{MediaSource, MediaSourceStream},
	sample::Sample,
};

use crate::{dsp::Frame, sound::static_sound::StaticSoundSettings};

use super::StaticSoundData;

impl StaticSoundData {
	fn load_from_media_source(
		media_source: Box<dyn MediaSource>,
		settings: StaticSoundSettings,
	) -> Result<Self, FromFileError> {
		let codecs = symphonia::default::get_codecs();
		let probe = symphonia::default::get_probe();
		let mss = MediaSourceStream::new(media_source, Default::default());
		let mut format_reader = probe
			.format(
				&Default::default(),
				mss,
				&Default::default(),
				&Default::default(),
			)?
			.format;
		let codec_params = &format_reader
			.default_track()
			.ok_or(FromFileError::NoDefaultTrack)?
			.codec_params;
		let sample_rate = codec_params
			.sample_rate
			.ok_or(FromFileError::UnknownSampleRate)?;
		let mut decoder = codecs.make(codec_params, &Default::default())?;
		let mut frames = vec![];
		loop {
			match format_reader.next_packet() {
				Ok(packet) => {
					let buffer = decoder.decode(&packet)?;
					load_frames_from_buffer_ref(&mut frames, &buffer)?;
				}
				Err(error) => match error {
					symphonia::core::errors::Error::IoError(error) => {
						if error.kind() == std::io::ErrorKind::UnexpectedEof {
							break;
						}
						return Err(symphonia::core::errors::Error::IoError(error).into());
					}
					error => return Err(error.into()),
				},
			}
		}
		Ok(Self {
			sample_rate,
			frames: Arc::new(frames),
			settings,
		})
	}

	/// Loads an audio file into a [`StaticSoundData`].
	#[cfg(not(target_arch = "wasm32"))]
	pub fn load(
		path: impl AsRef<Path>,
		settings: StaticSoundSettings,
	) -> Result<Self, FromFileError> {
		Self::load_from_media_source(Box::new(File::open(path)?), settings)
	}

	/// Loads a cursor wrapping audio file data into a [`StaticSoundData`].
	pub fn load_from_cursor<T: AsRef<[u8]> + Send + 'static>(
		cursor: Cursor<T>,
		settings: StaticSoundSettings,
	) -> Result<StaticSoundData, FromFileError> {
		Self::load_from_media_source(Box::new(cursor), settings)
	}
}

fn load_frames_from_buffer_ref(
	frames: &mut Vec<Frame>,
	buffer: &AudioBufferRef,
) -> Result<(), FromFileError> {
	match buffer {
		AudioBufferRef::U8(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::U16(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::U24(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::U32(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S8(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S16(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S24(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::S32(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::F32(buffer) => load_frames_from_buffer(frames, buffer),
		AudioBufferRef::F64(buffer) => load_frames_from_buffer(frames, buffer),
	}
}

fn load_frames_from_buffer<S: Sample>(
	frames: &mut Vec<Frame>,
	buffer: &AudioBuffer<S>,
) -> Result<(), FromFileError>
where
	f32: FromSample<S>,
{
	match buffer.spec().channels.count() {
		1 => {
			for sample in buffer.chan(0) {
				frames.push(Frame::from_mono((*sample).into_sample()));
			}
		}
		2 => {
			for (left, right) in buffer.chan(0).iter().zip(buffer.chan(1).iter()) {
				frames.push(Frame::new((*left).into_sample(), (*right).into_sample()));
			}
		}
		_ => return Err(FromFileError::UnsupportedChannelConfiguration),
	}
	Ok(())
}
