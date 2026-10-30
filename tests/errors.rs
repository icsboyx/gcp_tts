//! Error cases that are detected before any network request is sent.

mod common;

use common::italian_voice;
use gcp_tts::{
    GCP_TTS_SYNTHESIS_MAX_TEXT_LENGTH, GcpApiErrorKind, GcpTTS,
    audio_config::AudioConfig,
    speech_config::SpeechConfig,
    voices::Voices,
};

fn speech_config() -> SpeechConfig {
    SpeechConfig::new(italian_voice(), AudioConfig::default())
}

#[tokio::test]
async fn text_over_limit_is_rejected() {
    let tts = GcpTTS::new("unused-key");
    let text = "a".repeat(GCP_TTS_SYNTHESIS_MAX_TEXT_LENGTH + 1);

    let error = tts.synthesize_speech(&text, speech_config()).await.unwrap_err();

    assert_eq!(error.kind(), GcpApiErrorKind::InvalidInput);
    assert_eq!(error.to_string(), error.message());
}

#[tokio::test]
async fn text_limit_counts_bytes_not_characters() {
    let tts = GcpTTS::new("unused-key");
    // 'è' is 2 bytes in UTF-8: fewer characters than the limit, more bytes.
    let text = "è".repeat(GCP_TTS_SYNTHESIS_MAX_TEXT_LENGTH / 2 + 1);
    assert!(text.chars().count() < GCP_TTS_SYNTHESIS_MAX_TEXT_LENGTH);

    let error = tts.synthesize_speech(&text, speech_config()).await.unwrap_err();

    assert_eq!(error.kind(), GcpApiErrorKind::InvalidInput);
}

#[tokio::test]
async fn invalid_api_key_is_rejected_by_synthesize_speech() {
    // A newline is not allowed in an HTTP header value.
    let tts = GcpTTS::new("bad\nkey");

    let error = tts.synthesize_speech("Ciao", speech_config()).await.unwrap_err();

    assert_eq!(error.kind(), GcpApiErrorKind::InvalidInput);
}

#[tokio::test]
async fn invalid_api_key_is_rejected_by_get_voices() {
    let tts = GcpTTS::new("bad\nkey");

    let error = tts.get_voices().await.unwrap_err();

    assert_eq!(error.kind(), GcpApiErrorKind::InvalidInput);
}

#[tokio::test]
async fn voice_without_language_code_is_rejected() {
    let json = r#"{ "voices": [
        { "languageCodes": [], "name": "NoLanguage", "ssmlGender": "FEMALE", "naturalSampleRateHertz": 24000 }
    ] }"#;
    let voices: Voices = serde_json::from_str(json).unwrap();
    let config = SpeechConfig::new(voices.as_slice()[0].clone(), AudioConfig::default());

    let error = GcpTTS::new("unused-key")
        .synthesize_speech("Ciao", config)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), GcpApiErrorKind::InvalidInput);
}
