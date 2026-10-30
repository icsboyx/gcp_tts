//! Output audio settings.

use serde::{Deserialize, Serialize};

/// Format of the returned audio. Defaults to [`AudioEncoding::Mp3`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AudioEncoding {
    #[serde(rename = "AUDIO_ENCODING_UNSPECIFIED")]
    Unspecified,

    #[serde(rename = "LINEAR16")]
    Linear16,

    #[default]
    #[serde(rename = "MP3")]
    Mp3,

    #[serde(rename = "OGG_OPUS")]
    OggOpus,

    #[serde(rename = "MULAW")]
    Mulaw,

    #[serde(rename = "ALAW")]
    Alaw,

    #[serde(rename = "PCM")]
    Pcm,

    #[serde(rename = "M4A")]
    M4a,
}

impl AsRef<str> for AudioEncoding {
    fn as_ref(&self) -> &str {
        match self {
            Self::Unspecified => "AUDIO_ENCODING_UNSPECIFIED",
            Self::Linear16 => "LINEAR16",
            Self::Mp3 => "MP3",
            Self::OggOpus => "OGG_OPUS",
            Self::Mulaw => "MULAW",
            Self::Alaw => "ALAW",
            Self::Pcm => "PCM",
            Self::M4a => "M4A",
        }
    }
}

impl From<&str> for AudioEncoding {
    fn from(value: &str) -> Self {
        match value {
            "LINEAR16" => Self::Linear16,
            "MP3" => Self::Mp3,
            "OGG_OPUS" => Self::OggOpus,
            "MULAW" => Self::Mulaw,
            "ALAW" => Self::Alaw,
            "PCM" => Self::Pcm,
            "M4A" => Self::M4a,
            _ => Self::Unspecified,
        }
    }
}

impl From<String> for AudioEncoding {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl std::fmt::Display for AudioEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

/// Audio effects profile that optimizes the output for a class of device.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum EffectsProfile {
    #[serde(rename = "wearable-class-device")]
    WearableClassDevice,
    #[serde(rename = "handset-class-device")]
    HandsetClassDevice,
    #[serde(rename = "headphone-class-device")]
    HeadphoneClassDevice,
    #[serde(rename = "small-bluetooth-speaker-class-device")]
    SmallBluetoothSpeakerClassDevice,
    #[serde(rename = "medium-bluetooth-speaker-class-device")]
    MediumBluetoothSpeakerClassDevice,
    #[serde(rename = "large-home-entertainment-class-device")]
    LargeHomeEntertainmentClassDevice,
    #[serde(rename = "large-automotive-class-device")]
    LargeAutomotiveClassDevice,
    #[serde(rename = "telephony-class-application")]
    TelephonyClassApplication,
}

/// Audio settings for a synthesis request.
///
/// [`AudioConfig::default()`] gives MP3 with the voice's native rate, pitch and volume.
/// Setters can be chained:
///
/// ```
/// use gcp_tts::audio_config::{AudioConfig, AudioEncoding};
///
/// let mut config = AudioConfig::default();
/// config
///     .set_audio_encoding(AudioEncoding::OggOpus)
///     .set_speaking_rate(1.2)
///     .set_pitch(-2.0);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AudioConfig {
    audio_encoding: AudioEncoding,
    // Any other values < 0.25 or > 2.0 will return an error.
    speaking_rate: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pitch: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volume_gain_db: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sample_rate_hertz: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effects_profile_id: Option<Vec<EffectsProfile>>,
}

impl AudioConfig {
    /// Required. The format of the audio byte stream.
    pub fn set_audio_encoding(&mut self, audio_encoding: AudioEncoding) -> &mut Self {
        self.audio_encoding = audio_encoding;
        self
    }

    /// Optional. Input only. Speaking rate/speed, in the range [0.25, 2.0].
    /// 1.0 is the normal native speed supported by the specific voice. 2.0 is twice as fast, and 0.5 is half as fast.
    /// If unset(0.0), defaults to the native 1.0 speed. Any other values < 0.25 or > 2.0 will return an error.
    /// Clamps the value to the range [0.25, 2.0].
    pub fn set_speaking_rate(&mut self, speaking_rate: f32) -> &mut Self {
        self.speaking_rate = speaking_rate.clamp(0.25, 2.0);
        self
    }

    /// Optional. Input only. Speaking pitch, in the range [-20.0, 20.0].
    /// 20 means increase 20 semitones from the original pitch. -20 means decrease 20 semitones from the original pitch.
    /// Clamps the value to the range [-20.0, 20.0].
    pub fn set_pitch(&mut self, pitch: f32) -> &mut Self {
        self.pitch = Some(pitch.clamp(-20.0, 20.0));
        self
    }

    /// Optional. Input only. Volume gain (in dB) of the normal native volume supported by the specific voice,
    /// in the range [-96.0, 16.0]. If unset, or set to a value of 0.0 (dB), will play at normal native signal amplitude.
    /// A value of -6.0 (dB) will play at approximately half the amplitude of the normal native signal amplitude.
    /// A value of +6.0 (dB) will play at approximately twice the amplitude of the normal native signal amplitude.
    /// Strongly recommend not to exceed +10 (dB) as there's usually no effective increase in loudness for any value greater than that.
    /// Clamping the value to the range [-96.0, 10.0].
    pub fn set_volume_gain_db(&mut self, volume_gain_db: f32) -> &mut Self {
        self.volume_gain_db = Some(volume_gain_db.clamp(-96.0, 10.0));
        self
    }

    /// Optional. The synthesis sample rate (in hertz) for this audio. When this is specified in SynthesizeSpeechRequest,
    /// if this is different from the voice's natural sample rate, then the synthesizer
    /// will honor this request by converting to the desired sample rate (which might result in worse audio quality),
    /// unless the specified sample rate is not supported for the encoding chosen, in which case it will fail the request and return
    /// google.rpc.Code.INVALID_ARGUMENT.
    /// (Error is generated during the audio decoding inside GCP, if an unsupported sample rate is requested.)
    pub fn set_sample_rate_hertz(&mut self, sample_rate_hertz: u32) -> &mut Self {
        self.sample_rate_hertz = Some(sample_rate_hertz);
        self
    }
    /// Optional. Input only. An identifier which selects 'audio effects' profiles that are applied on (post synthesized) text to speech.
    /// Effects are applied on top of each other in the order they are given. See audio profiles for current supported profile ids.
    pub fn set_effects_profile_id(&mut self, effects_profile_id: Vec<EffectsProfile>) -> &mut Self {
        self.effects_profile_id = Some(effects_profile_id);
        self
    }
}
