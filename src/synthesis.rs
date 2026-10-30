//! Request and response types of the `text:synthesize` endpoint.
//!
//! Most users only need [`GcpTTS::synthesize_speech`](crate::GcpTTS::synthesize_speech).

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

use crate::{
    GcpApiError,
    audio_config::AudioConfig,
    voices::{SsmlGender, Voice},
};

/// Endpoint of the `text:synthesize` method.
pub static GCP_SYNTHESIS_URL: &str = "https://texttospeech.googleapis.com/v1/text:synthesize";

/// Voice selection sent to the API. Build it from a [`Voice`] with `try_from`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSelectionParams {
    pub language_code: String,
    pub name: String,
    pub ssml_gender: SsmlGender,
}

impl TryFrom<&Voice> for VoiceSelectionParams {
    type Error = GcpApiError;
    fn try_from(voice: &Voice) -> Result<Self, Self::Error> {
        Ok(VoiceSelectionParams {
            language_code: voice
                .language_codes
                .first()
                .ok_or("Unable to get language code")?
                .to_string(),
            name: voice.name.clone(),
            ssml_gender: voice.gender.clone(),
        })
    }
}

/// JSON body of a `text:synthesize` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynthesisPayload {
    input: SynthesisInput,
    #[serde(rename = "voice")]
    voice_selection_params: VoiceSelectionParams,
    audio_config: AudioConfig,
}

impl SynthesisPayload {
    /// Payload for plain text input.
    ///
    /// Fails if `voice` has no language code.
    pub fn text(
        text: impl AsRef<str>,
        voice: &Voice,
        audio_config: AudioConfig,
    ) -> Result<Self, GcpApiError> {
        let voice_param = VoiceSelectionParams::try_from(voice)?;
        Ok(SynthesisPayload {
            input: SynthesisInput::Text(text.as_ref().to_string()),
            voice_selection_params: voice_param,
            audio_config,
        })
    }

    /// Payload for markup input (supported only by some voices).
    pub fn markup(
        markup: impl AsRef<str>,
        voice_selection_params: VoiceSelectionParams,
        audio_config: AudioConfig,
    ) -> Self {
        SynthesisPayload {
            input: SynthesisInput::Markup(markup.as_ref().to_string()),
            voice_selection_params,
            audio_config,
        }
    }

    /// Payload for SSML input.
    pub fn ssml(
        ssml: impl AsRef<str>,
        voice_selection_params: VoiceSelectionParams,
        audio_config: AudioConfig,
    ) -> Self {
        SynthesisPayload {
            input: SynthesisInput::Ssml(ssml.as_ref().to_string()),
            voice_selection_params,
            audio_config,
        }
    }
}

/// Supporting only text, markup and ssml for now.
/// <https://docs.cloud.google.com/text-to-speech/docs/reference/rest/v1/SynthesisInput>
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SynthesisInput {
    Text(String),
    Markup(String),
    Ssml(String),
}

/// JSON body of a successful `text:synthesize` response.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynthesisResponse {
    audio_content: String,
}

impl SynthesisResponse {
    /// Decodes the base64 audio returned by the API into raw bytes.
    pub fn audio_content(&self) -> Result<Vec<u8>, GcpApiError> {
        let audio_data = STANDARD.decode(self.audio_content.clone())?;
        Ok(audio_data)
    }
}
