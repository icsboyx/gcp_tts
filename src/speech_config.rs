//! Settings for a single synthesis request.

use crate::{audio_config::AudioConfig, voices::Voice};
use serde::{Deserialize, Serialize};

/// Voice and audio settings passed to [`GcpTTS::synthesize_speech`](crate::GcpTTS::synthesize_speech).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechConfig {
    voice: Voice,
    audio_config: AudioConfig,
}

impl SpeechConfig {
    /// Combines a voice (usually taken from [`Voices`](crate::voices::Voices)) with audio settings.
    pub fn new(voice: Voice, audio_config: AudioConfig) -> Self {
        Self {
            voice,
            audio_config,
        }
    }

    /// Returns the selected voice.
    pub fn voice(&self) -> &Voice {
        &self.voice
    }

    /// Returns the audio settings.
    pub fn audio_config(&self) -> &AudioConfig {
        &self.audio_config
    }
}
