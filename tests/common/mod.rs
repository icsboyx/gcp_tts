use gcp_tts::voices::{Voice, Voices};

/// Voice list in the same JSON shape returned by the `voices.list` endpoint.
pub const VOICES_JSON: &str = r#"{
  "voices": [
    { "languageCodes": ["it-IT"], "name": "it-IT-Standard-A", "ssmlGender": "FEMALE", "naturalSampleRateHertz": 24000 },
    { "languageCodes": ["it-IT"], "name": "it-IT-Neural2-C", "ssmlGender": "MALE", "naturalSampleRateHertz": 24000 },
    { "languageCodes": ["en-US"], "name": "en-US-Chirp3-HD-Achernar", "ssmlGender": "FEMALE", "naturalSampleRateHertz": 24000 },
    { "languageCodes": ["en-US"], "name": "Achird", "ssmlGender": "MALE", "naturalSampleRateHertz": 24000 }
  ]
}"#;

pub fn sample_voices() -> Voices {
    serde_json::from_str(VOICES_JSON).expect("sample voices must deserialize")
}

#[allow(dead_code)] // not every test file uses it
pub fn italian_voice() -> Voice {
    sample_voices()
        .get_voice_by_name("it-IT-Standard-A")
        .expect("sample contains it-IT-Standard-A")
        .clone()
}
