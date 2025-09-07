use std::{fs::File, path::Path};

use anyhow::{anyhow, bail, Result};
use ndarray::{Array2, ArrayView2};
use rubato::{FftFixedInOut, Resampler};
use symphonia::core::{
    audio::{AudioBufferRef, Signal},
    codecs::CODEC_TYPE_NULL,
    conv::IntoSample,
    errors::Error as SymphoniaError,
    io::MediaSourceStream,
    probe::Hint,
};

const SAMPLE_RATE: u32 = 44100;

#[tracing::instrument(skip_all)]
fn resample(samples: Vec<Vec<f64>>, original_sample_rate: u32) -> Result<Vec<Vec<f64>>> {
    tracing::info!(sample_rate = original_sample_rate, "Start resampling...");

    let channels = samples.len();
    let nbr_input_frames = samples[0].len();

    let f_ratio = SAMPLE_RATE as f64 / original_sample_rate as f64;

    let mut outdata =
        vec![Vec::with_capacity((nbr_input_frames as f64 * f_ratio) as usize); channels];

    let mut resampler = FftFixedInOut::<f64>::new(
        original_sample_rate as usize,
        SAMPLE_RATE as usize,
        1024, // TODO: maybe adjust this?
        channels,
    )?;

    let mut input_frames_next = resampler.input_frames_next();
    let mut outbuffer = vec![vec![0.0; resampler.output_frames_max()]; channels];
    let mut indata_slices: Vec<&[f64]> = samples.iter().map(|v| &v[..]).collect();

    fn append_frames(buffers: &mut [Vec<f64>], additional: &[Vec<f64>], nbr_frames: usize) {
        buffers
            .iter_mut()
            .zip(additional.iter())
            .for_each(|(b, a)| b.extend_from_slice(&a[..nbr_frames]));
    }

    while indata_slices[0].len() >= input_frames_next {
        let (nbr_in, nbr_out) =
            resampler.process_into_buffer(&indata_slices, &mut outbuffer, None)?;

        for chan in indata_slices.iter_mut() {
            *chan = &chan[nbr_in..];
        }

        append_frames(&mut outdata, &outbuffer, nbr_out);
        input_frames_next = resampler.input_frames_next();
    }

    if !indata_slices[0].is_empty() {
        let (_, nbr_out) =
            resampler.process_partial_into_buffer(Some(&indata_slices), &mut outbuffer, None)?;

        append_frames(&mut outdata, &outbuffer, nbr_out);
    }

    tracing::info!(samples = outdata[0].len(), "Audio resampled");

    Ok(outdata)
}

#[tracing::instrument(skip_all)]
pub fn read_audio(path: impl AsRef<Path>) -> Result<Array2<f64>> {
    let path = path.as_ref();
    let src = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let probed = symphonia::default::get_probe().format(
        &Hint::new(),
        mss,
        &Default::default(),
        &Default::default(),
    )?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| anyhow!("No supported audio tracks"))?;

    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &Default::default())?;

    let mut samples: Vec<Vec<f64>> = Vec::new();
    let mut sample_rate = track.codec_params.sample_rate;

    let track_id = track.id;

    tracing::info!("Start decoding...");

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::ResetRequired) => {
                unimplemented!();
            }
            Err(SymphoniaError::IoError(err))
                if err.kind() == std::io::ErrorKind::UnexpectedEof
                    && err.to_string() == "end of stream" =>
            {
                break;
            }
            Err(err) => {
                bail!(err);
            }
        };

        if packet.track_id() != track_id {
            tracing::warn!(
                timestamp = packet.ts,
                "The packet does not belong to the selected track, skip..."
            );
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = decoded.spec();

                if sample_rate.is_none() {
                    sample_rate = Some(spec.rate);
                }

                let channel_num = spec.channels.count();

                if samples.len() < channel_num {
                    samples.resize_with(channel_num, Vec::new);
                }

                macro_rules! copy_samples {
          ($(($enum:ident, $type:ty)),*) => {
            match &decoded {
              $(
                AudioBufferRef::$enum(buf) => {
                  let f = |&v| <$type as IntoSample<f64>>::into_sample(v);
                  for ch in 0..channel_num {
                    samples[ch].extend(buf.chan(ch).iter().map(f));
                  }
                }
              )*
              AudioBufferRef::F64(buf) => {
                for ch in 0..channel_num {
                  samples[ch].extend_from_slice(buf.chan(ch));
                }
              }
            }
          }
        }

                copy_samples!(
                    (U8, u8),
                    (U16, u16),
                    (U24, symphonia::core::sample::u24),
                    (U32, u32),
                    (S8, i8),
                    (S16, i16),
                    (S24, symphonia::core::sample::i24),
                    (S32, i32),
                    (F32, f32)
                );
            }
            Err(SymphoniaError::IoError(_)) => {
                tracing::error!(
                    timestamp = packet.ts,
                    "The packet failed to decode due to an IO error, skip..."
                );
                continue;
            }
            Err(SymphoniaError::DecodeError(_)) => {
                tracing::warn!(
                    timestamp = packet.ts,
                    "The packet failed to decode due to invalid data, skip..."
                );
                continue;
            }
            Err(err) => {
                bail!(err);
            }
        }
    }

    tracing::info!(samples = samples[0].len(), "Audio decoded");

    let sample_rate = sample_rate.ok_or_else(|| anyhow!("Can not get sample rate"))?;

    if sample_rate != SAMPLE_RATE {
        samples = resample(samples, sample_rate)?;
    }

    let channel_num = samples.len();
    let length = samples
        .iter()
        .map(|c| c.len())
        .max()
        .ok_or_else(|| anyhow!("No channel found"))?;

    let res = Array2::from_shape_vec(
        (channel_num, length),
        samples.into_iter().flatten().collect(),
    )?;

    tracing::info!(?path, "Audio read");

    Ok(res)
}

#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    Wav,
    Flac,
}

impl AudioFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Flac => "flac",
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn write_audio(
    path: impl AsRef<Path>,
    audio: ArrayView2<f64>,
    format: &AudioFormat,
) -> Result<()> {
    let path = path.as_ref();

    match format {
        AudioFormat::Wav => write_wav(path, audio),
        AudioFormat::Flac => write_flac(path, audio),
    }
}

fn write_wav(path: &Path, audio: ArrayView2<f64>) -> Result<()> {
    let (channel_num, _) = audio.dim();

    let spec = hound::WavSpec {
        channels: channel_num.try_into()?,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)?;

    for &sample in audio.t().iter() {
        let sample: i16 = sample.into_sample();
        writer.write_sample(sample)?;
    }

    writer.finalize()?;

    tracing::info!(?path, "Audio has been written");

    Ok(())
}

fn write_flac(path: &Path, audio: ArrayView2<f64>) -> Result<()> {
    use libflac::Encoder;

    let (channel_num, _) = audio.dim();

    let encoder = Encoder::new()
        .set_channels(channel_num as u32)
        .set_bits_per_sample(16)
        .set_sample_rate(SAMPLE_RATE)
        .set_compression_level(8)
        .init_file(path)?;

    let data: Vec<i16> = audio.t().into_iter().map(|&s| s.into_sample()).collect();

    encoder.process_interleaved(&data)?.finish()?;

    tracing::info!(?path, "Audio has been written");

    Ok(())
}
