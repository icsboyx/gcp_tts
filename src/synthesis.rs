//! Request and response types of the `text:synthesize` endpoint.
//!
//! Internal: users call [`GcpTTS::synthesize_speech`](crate::GcpTTS::synthesize_speech).

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

use crate::{
    GcpApiError,
    audio_config::AudioConfig,
    voices::{SsmlGender, Voice},
};

/// Endpoint of the `text:synthesize` method.
pub(crate) static GCP_SYNTHESIS_URL: &str =
    "https://texttospeech.googleapis.com/v1/text:synthesize";

/// Voice selection sent to the API. Build it from a [`Voice`] with `try_from`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VoiceSelectionParams {
    language_code: String,
    name: String,
    ssml_gender: SsmlGender,
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
pub(crate) struct SynthesisPayload {
    input: SynthesisInput,
    #[serde(rename = "voice")]
    voice_selection_params: VoiceSelectionParams,
    audio_config: AudioConfig,
}

impl SynthesisPayload {
    /// Payload for plain text input.
    ///
    /// Fails if `voice` has no language code.
    pub(crate) fn text(
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

    /// Copy of the payload with the input replaced by its size, safe to put in
    /// error messages and logs.
    pub(crate) fn redacted(&self) -> Self {
        let size = |input: &str| format!("<{} bytes redacted>", input.len());
        let input = match &self.input {
            SynthesisInput::Text(text) => SynthesisInput::Text(size(text)),
            SynthesisInput::Markup(markup) => SynthesisInput::Markup(size(markup)),
            SynthesisInput::Ssml(ssml) => SynthesisInput::Ssml(size(ssml)),
        };
        SynthesisPayload {
            input,
            voice_selection_params: self.voice_selection_params.clone(),
            audio_config: self.audio_config.clone(),
        }
    }

    /// Payload for markup input (supported only by some voices).
    #[allow(dead_code)] // Not exposed by `GcpTTS` yet: see TODO.md
    pub(crate) fn markup(
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
    #[allow(dead_code)] // Not exposed by `GcpTTS` yet: see TODO.md
    pub(crate) fn ssml(
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
pub(crate) enum SynthesisInput {
    Text(String),
    Markup(String),
    Ssml(String),
}

/// JSON body of a successful `text:synthesize` response.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SynthesisResponse {
    audio_content: String,
}

impl SynthesisResponse {
    /// Decodes the base64 audio returned by the API into raw bytes.
    pub(crate) fn audio_content(&self) -> Result<Vec<u8>, GcpApiError> {
        let audio_data = STANDARD.decode(self.audio_content.clone())?;
        Ok(audio_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GcpApiErrorKind;
    use serde_json::json;

    fn italian_voice() -> Voice {
        serde_json::from_str(
            r#"{ "languageCodes": ["it-IT"], "name": "it-IT-Standard-A", "ssmlGender": "FEMALE", "naturalSampleRateHertz": 24000 }"#,
        )
        .unwrap()
    }

    #[test]
    fn text_payload_has_api_shape() {
        let payload =
            SynthesisPayload::text("Ciao", &italian_voice(), AudioConfig::default()).unwrap();

        let value = serde_json::to_value(&payload).unwrap();

        assert_eq!(
            value,
            json!({
                "input": { "text": "Ciao" },
                "voice": {
                    "languageCode": "it-IT",
                    "name": "it-IT-Standard-A",
                    "ssmlGender": "FEMALE"
                },
                "audioConfig": { "audioEncoding": "MP3", "speakingRate": 0.0 }
            })
        );
    }

    #[test]
    fn ssml_and_markup_payloads_use_their_input_key() {
        let voice = VoiceSelectionParams::try_from(&italian_voice()).unwrap();

        let ssml =
            SynthesisPayload::ssml("<speak>Ciao</speak>", voice.clone(), AudioConfig::default());
        let markup = SynthesisPayload::markup("Ciao", voice, AudioConfig::default());

        assert_eq!(
            serde_json::to_value(&ssml).unwrap()["input"],
            json!({ "ssml": "<speak>Ciao</speak>" })
        );
        assert_eq!(
            serde_json::to_value(&markup).unwrap()["input"],
            json!({ "markup": "Ciao" })
        );
    }

    #[test]
    fn voice_without_language_code_is_rejected() {
        let mut voice = italian_voice();
        voice.language_codes.clear();

        let error = VoiceSelectionParams::try_from(&voice).unwrap_err();
        assert_eq!(error.kind(), GcpApiErrorKind::InvalidInput);
    }

    #[test]
    fn redacted_payload_hides_input_text() {
        let payload =
            SynthesisPayload::text("Ciao", &italian_voice(), AudioConfig::default()).unwrap();

        let value = serde_json::to_value(payload.redacted()).unwrap();

        assert_eq!(value["input"], json!({ "text": "<4 bytes redacted>" }));
        assert_eq!(value["voice"]["name"], "it-IT-Standard-A");
    }

    #[test]
    fn synthesis_response_decodes_base64_audio() {
        // "aGVsbG8=" is base64 for "hello".
        let response: SynthesisResponse =
            serde_json::from_str(r#"{ "audioContent": "aGVsbG8=" }"#).unwrap();

        assert_eq!(response.audio_content().unwrap(), b"hello");
    }

    #[test]
    fn invalid_base64_audio_is_an_audio_error() {
        let response: SynthesisResponse =
            serde_json::from_str(r#"{ "audioContent": "@@not-base64@@" }"#).unwrap();

        let error = response.audio_content().unwrap_err();

        assert_eq!(error.kind(), GcpApiErrorKind::AudioError);
    }
}
