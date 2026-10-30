use gcp_tts::audio_config::{AudioConfig, AudioEncoding, EffectsProfile};
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
