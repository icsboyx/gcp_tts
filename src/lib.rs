//! Minimal async client for the Google Cloud Text-to-Speech REST API.
//!
//! Authentication uses an API key, sent in the `x-goog-api-key` header.
//!
//! # Example
//!
//! ```no_run
//! use gcp_tts::{
//!     GcpTTS, audio_config::AudioConfig, speech_config::SpeechConfig, voices::LanguageCodes,
//! };
//!
//! # async fn run() -> Result<(), gcp_tts::GcpApiError> {
//! let tts = GcpTTS::new("YOUR_API_KEY");
//!
//! // Download the voice list once, then filter it locally.
//! let voices = tts.get_voices().await?;
//! let voice = voices
//!     .filter_by_language_codes(LanguageCodes::Italian)
//!     .random_voice()
//!     .expect("no Italian voice available")
//!     .clone();
//!
//! // AudioConfig::default() produces MP3.
//! let config = SpeechConfig::new(voice, AudioConfig::default());
//! let audio = tts.synthesize_speech("Ciao a tutti!", config).await?;
//! std::fs::write("output.mp3", audio).expect("failed to write file");
//! # Ok(())
//! # }
//! ```

use crate::{
    GcpApiErrorKind::{GcpApi, InvalidInput},
    speech_config::SpeechConfig,
    synthesis::{GCP_SYNTHESIS_URL, SynthesisResponse},
    voices::Voices,
};

pub mod audio_config;
pub mod speech_config;
mod synthesis;
pub mod voices;

/// Maximum size of the input text accepted by GCP in a single request, in bytes.
///
/// The limit is set by Google and may change.
pub static GCP_TTS_SYNTHESIS_MAX_TEXT_LENGTH: usize = 5000;

/// Text-to-Speech client.
///
/// Holds the API key and a single HTTP client, so create it once and reuse it:
/// connections are kept alive between requests.
pub struct GcpTTS {
    api_key: String,
    http_client: reqwest::Client,
}

impl GcpTTS {
    /// Creates a client that authenticates with the given GCP API key.
    pub fn new(api_key: impl AsRef<str>) -> Self {
        GcpTTS {
            api_key: api_key.as_ref().to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Downloads the list of all available voices.
    ///
    /// Each call is a network request: call it once and use the
    /// [`Voices`] filters to pick a voice.
    pub async fn get_voices(&self) -> Result<Voices, GcpApiError> {
        let voices = Voices::get_voices(&self.api_key, &self.http_client).await?;
        Ok(voices)
    }

    /// Synthesizes `text` and returns the raw audio bytes.
    ///
    /// The audio format is the one set in the [`AudioConfig`](audio_config::AudioConfig)
    /// of `speech_config` (MP3 by default), ready to be written to a file.
    ///
    /// Input larger than [`GCP_TTS_SYNTHESIS_MAX_TEXT_LENGTH`] bytes is rejected
    /// before sending the request. Synthesis is billed per character.
    ///
    /// # Errors
    ///
    /// - [`GcpApiErrorKind::InvalidInput`]: text too long, voice without a language
    ///   code, or API key with characters not allowed in an HTTP header.
    /// - [`GcpApiErrorKind::Network`]: the request could not be sent or the
    ///   response could not be read.
    /// - [`GcpApiErrorKind::GcpApi`]: GCP answered with a non-success HTTP status,
    ///   available from [`GcpApiError::status`].
    /// - [`GcpApiErrorKind::InvalidResponse`]: the response body is not valid JSON
    ///   of the expected shape.
    /// - [`GcpApiErrorKind::AudioError`]: the returned audio is not valid base64.
    pub async fn synthesize_speech(
        &self,
        text: &str,
        speech_config: SpeechConfig,
    ) -> Result<Vec<u8>, GcpApiError> {
        if text.len() > GCP_TTS_SYNTHESIS_MAX_TEXT_LENGTH {
            return Err(GcpApiError::new(
                InvalidInput,
                format!(
                    "Text to synthesize is more than {GCP_TTS_SYNTHESIS_MAX_TEXT_LENGTH} bytes."
                ),
            ));
        }

        let payload = synthesis::SynthesisPayload::text(
            text,
            speech_config.voice(),
            speech_config.audio_config().clone(),
        )?;

        let mut header = reqwest::header::HeaderMap::new();
        header.insert("Content-Type", "application/json".parse()?);
        header.insert("x-goog-api-key", self.api_key.parse()?);

        let request = self
            .http_client
            .post(GCP_SYNTHESIS_URL)
            .headers(header)
            .json(&payload);

        let request_debug = format!(
            "POST {GCP_SYNTHESIS_URL}\nPayload:\n{}",
            serde_json::to_string_pretty(&payload.redacted())?
        );

        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(format_api_error(request_debug, response).await?);
        }

        response.json::<SynthesisResponse>().await?.audio_content()
    }
}

/// Category of a [`GcpApiError`], to decide how to react to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GcpApiErrorKind {
    /// Invalid input from the caller: text over the size limit, voice without
    /// a language code, or an API key with characters not allowed in an HTTP header.
    InvalidInput,
    /// An endpoint URL could not be parsed. Should never happen: the URLs are constants.
    Url,
    /// The request could not be sent or the response could not be read:
    /// DNS, connection, TLS or a dropped connection. Usually worth retrying.
    Network,
    /// GCP answered with a non-success HTTP status. The message contains the
    /// GCP error and [`GcpApiError::status`] the HTTP status code: retry on
    /// `429` and `5xx`, fix the request or the project setup on other `4xx`.
    GcpApi,
    /// The response body is not valid JSON of the expected shape.
    InvalidResponse,
    /// JSON serialization failed while building an error message.
    /// Should never happen with this crate's types.
    SerdeJson,
    /// The audio returned by GCP is not valid base64.
    AudioError,
}

/// Error returned by every fallible operation of this crate.
///
/// Use [`kind`](GcpApiError::kind) to decide how to react and
/// [`Display`](std::fmt::Display) to show a human-readable message.
pub struct GcpApiError {
    kind: GcpApiErrorKind,
    message: String,
    status: Option<u16>,
}

impl GcpApiError {
    pub(crate) fn new(kind: GcpApiErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            status: None,
        }
    }

    /// Returns the error category.
    pub fn kind(&self) -> GcpApiErrorKind {
        self.kind
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the HTTP status code sent by GCP.
    ///
    /// Set only for [`GcpApiErrorKind::GcpApi`] errors.
    pub fn status(&self) -> Option<u16> {
        self.status
    }
}

impl std::error::Error for GcpApiError {}

impl std::fmt::Display for GcpApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

macro_rules! impl_from_error {
    ($($type:ty => $kind:expr),+ $(,)?) => {
        $(
            impl From<$type> for GcpApiError {
                fn from(error: $type) -> Self {
                    GcpApiError::new($kind, error.to_string())
                }
            }
        )+
    };
}

impl From<reqwest::Error> for GcpApiError {
    fn from(error: reqwest::Error) -> Self {
        let kind = if error.is_decode() {
            GcpApiErrorKind::InvalidResponse
        } else {
            GcpApiErrorKind::Network
        };
        GcpApiError::new(kind, error.to_string())
    }
}

impl_from_error!(
    url::ParseError => GcpApiErrorKind::Url,
    reqwest::header::InvalidHeaderValue => GcpApiErrorKind::InvalidInput,
    serde_json::Error => GcpApiErrorKind::SerdeJson,
    base64::DecodeError => GcpApiErrorKind::AudioError,
    String => GcpApiErrorKind::InvalidInput,
    &str => GcpApiErrorKind::InvalidInput,
    &String => GcpApiErrorKind::InvalidInput,
);

async fn format_api_error(
    request_dbg: impl AsRef<str>,
    response: reqwest::Response,
) -> Result<GcpApiError, GcpApiError> {
    let status = response.status();
    let url = response.url().clone();
    let raw_body = response.text().await?;

    let json = serde_json::from_str::<serde_json::Value>(&raw_body).ok();

    let message = json
        .as_ref()
        .and_then(|value| value.pointer("/error/message"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("GCP did not provide an error message");

    let api_status = json
        .as_ref()
        .and_then(|value| value.pointer("/error/status"))
        .and_then(serde_json::Value::as_str);

    let formatted_body = match json.as_ref() {
        Some(value) => serde_json::to_string_pretty(value)?,
        None => raw_body,
    };

    let title = match api_status {
        Some(api_status) => {
            format!("GCP API request failed: HTTP {status} ({api_status})")
        }
        None => format!("GCP API request failed: HTTP {status}"),
    };

    let mut error = GcpApiError::new(
        GcpApi,
        format!(
            "{title}\n{message}\n\nRequest:\n{}\n\nResponse:\n  URL: {url}\n  Body:\n{formatted_body}",
            request_dbg.as_ref()
        ),
    );
    error.status = Some(status.as_u16());
    Ok(error)
}

impl std::fmt::Debug for GcpApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GcpApiError({})", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(status: u16, body: &'static str) -> reqwest::Response {
        http::Response::builder()
            .status(status)
            .body(body)
            .unwrap()
            .into()
    }

    #[tokio::test]
    async fn gcp_error_body_is_parsed() {
        let body = r#"{ "error": { "code": 429, "message": "Quota exceeded", "status": "RESOURCE_EXHAUSTED" } }"#;

        let error = format_api_error("POST test", response(429, body))
            .await
            .unwrap();

        assert_eq!(error.kind(), GcpApi);
        assert_eq!(error.status(), Some(429));
        assert!(error.message().contains("HTTP 429 Too Many Requests (RESOURCE_EXHAUSTED)"));
        assert!(error.message().contains("Quota exceeded"));
    }

    #[tokio::test]
    async fn non_json_error_body_keeps_status_and_text() {
        let error = format_api_error("POST test", response(502, "<html>Bad Gateway</html>"))
            .await
            .unwrap();

        assert_eq!(error.kind(), GcpApi);
        assert_eq!(error.status(), Some(502));
        assert!(error.message().contains("<html>Bad Gateway</html>"));
    }

    #[tokio::test]
    async fn undecodable_success_body_is_invalid_response() {
        let error: GcpApiError = response(200, "not json")
            .json::<SynthesisResponse>()
            .await
            .err()
            .unwrap()
            .into();

        assert_eq!(error.kind(), GcpApiErrorKind::InvalidResponse);
        assert_eq!(error.status(), None);
    }
}
