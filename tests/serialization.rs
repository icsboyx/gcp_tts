mod common;

use common::italian_voice;
use gcp_tts::{
    audio_config::{AudioConfig, AudioEncoding, EffectsProfile},
    synthesis::{SynthesisPayload, SynthesisResponse, VoiceSelectionParams},
};
use serde_json::json;

#[test]
fn default_audio_config_skips_unset_fields() {
    let value = serde_json::to_value(AudioConfig::default()).unwrap();

    assert_eq!(value, json!({ "audioEncoding": "MP3", "speakingRate": 0.0 }));
}

#[test]
fn audio_config_setters_serialize_with_api_names() {
    let mut config = AudioConfig::default();
    config
        .set_audio_encoding(AudioEncoding::OggOpus)
        .set_speaking_rate(1.5)
        .set_pitch(-2.0)
        .set_volume_gain_db(3.0)
        .set_sample_rate_hertz(48000)
        .set_effects_profile_id(vec![EffectsProfile::HeadphoneClassDevice]);

    let value = serde_json::to_value(&config).unwrap();

    assert_eq!(
        value,
        json!({
            "audioEncoding": "OGG_OPUS",
            "speakingRate": 1.5,
            "pitch": -2.0,
            "volumeGainDb": 3.0,
            "sampleRateHertz": 48000,
            "effectsProfileId": ["headphone-class-device"]
        })
    );
}

#[test]
fn audio_config_setters_clamp_out_of_range_values() {
    let mut config = AudioConfig::default();
    config
        .set_speaking_rate(5.0)
        .set_pitch(-50.0)
        .set_volume_gain_db(50.0);

    let value = serde_json::to_value(&config).unwrap();

    assert_eq!(value["speakingRate"], 2.0);
    assert_eq!(value["pitch"], -20.0);
    assert_eq!(value["volumeGainDb"], 10.0);
}

#[test]
fn text_payload_has_api_shape() {
    let payload = SynthesisPayload::text("Ciao", &italian_voice(), AudioConfig::default()).unwrap();

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

    let ssml = SynthesisPayload::ssml("<speak>Ciao</speak>", voice.clone(), AudioConfig::default());
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
fn synthesis_response_decodes_base64_audio() {
    // "aGVsbG8=" is base64 for "hello".
    let response: SynthesisResponse =
        serde_json::from_str(r#"{ "audioContent": "aGVsbG8=" }"#).unwrap();

    assert_eq!(response.audio_content().unwrap(), b"hello");
}
