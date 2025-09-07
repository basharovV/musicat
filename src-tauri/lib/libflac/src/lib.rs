mod error;

use std::{ffi::CString, mem::ManuallyDrop, path::Path, ptr::NonNull};

use libflac_sys::*;
use paste::paste;

use error::{Error, Result};

pub struct Encoder(NonNull<FLAC__StreamEncoder>);

impl Drop for Encoder {
  fn drop(&mut self) {
    unsafe { FLAC__stream_encoder_delete(self.0.as_ptr()) }
  }
}

macro_rules! set_settings {
  ($($setting:ident),*) => {
    $(
      paste! {
        pub fn [<set_ $setting>](self, $setting: u32) -> Self {
          let res = unsafe { [<FLAC__stream_encoder_set_ $setting>](self.0.as_ptr(), $setting) };
          if res == 0 {
            panic!("the encoder is already initialized");
          }
          self
        }
      }
    )*
  };
}

impl Encoder {
  pub fn new() -> Self {
    Default::default()
  }

  set_settings!(channels, bits_per_sample, sample_rate, compression_level);

  pub fn init_file(self, path: impl AsRef<Path>) -> Result<InitializedEncoder> {
    let path = path.as_ref();
    let raw_path = CString::new(path.as_os_str().as_encoded_bytes())
      .map_err(|_| Error::PathEncoding(path.to_owned()))?;
    let res = unsafe {
      FLAC__stream_encoder_init_file(
        self.0.as_ptr(),
        raw_path.as_ptr(),
        None,
        std::ptr::null_mut(),
      )
    };
    if res != FLAC__StreamEncoderInitStatus_FLAC__STREAM_ENCODER_INIT_STATUS_OK {
      return Err(Error::EncoderInitFailure(res));
    }
    let raw_self = ManuallyDrop::new(self);
    Ok(InitializedEncoder(raw_self.0))
  }
}

impl Default for Encoder {
  fn default() -> Self {
    Self(unsafe { NonNull::new_unchecked(FLAC__stream_encoder_new()) })
  }
}

pub struct InitializedEncoder(NonNull<FLAC__StreamEncoder>);

impl Drop for InitializedEncoder {
  fn drop(&mut self) {
    unsafe { FLAC__stream_encoder_delete(self.0.as_ptr()) }
  }
}

pub trait Sample {
  fn to_i32(self) -> i32;
}

impl Sample for i16 {
  fn to_i32(self) -> i32 {
    self.into()
  }
}

impl InitializedEncoder {
  pub fn process_interleaved(self, buffer: &[impl Sample + Clone]) -> Result<Self> {
    let channels = unsafe { FLAC__stream_encoder_get_channels(self.0.as_ptr()) };
    let data: Vec<_> = buffer.iter().map(|v| v.to_owned().to_i32()).collect();
    let res = unsafe {
      FLAC__stream_encoder_process_interleaved(
        self.0.as_ptr(),
        data.as_ptr(),
        data.len() as u32 / channels,
      )
    };
    if res == 0 {
      Err(Error::EncodeFailure)
    } else {
      Ok(self)
    }
  }

  pub fn finish(self) -> Result<Encoder> {
    let res = unsafe { FLAC__stream_encoder_finish(self.0.as_ptr()) };
    if res == 0 {
      Err(Error::EncodeFailure)
    } else {
      let raw_self = ManuallyDrop::new(self);
      Ok(Encoder(raw_self.0))
    }
  }
}
