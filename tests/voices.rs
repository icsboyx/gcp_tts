mod common;

use common::sample_voices;
use gcp_tts::voices::{LanguageCodes, SsmlGender, VoiceCategory, Voices};

#[test]
fn deserializes_voice_list() {
    let voices = sample_voices();

    assert_eq!(voices.len(), 4);
    let voice = voices.get_voice_by_name("it-IT-Standard-A").unwrap();
    assert_eq!(voice.language_codes, vec![LanguageCodes::Italian]);
    assert_eq!(voice.gender, SsmlGender::Female);
    assert_eq!(voice.natural_sample_rate_hertz, 24000);
}

#[test]
fn derives_category_from_voice_name() {
    let voices = sample_voices();
    let category_of = |name: &str| {
        voices
            .get_voice_by_name(name)
            .unwrap()
            .voice_category
            .clone()
    };

    assert_eq!(category_of("it-IT-Standard-A"), VoiceCategory::Standard);
    assert_eq!(category_of("it-IT-Neural2-C"), VoiceCategory::Neural2);
    assert_eq!(
        category_of("en-US-Chirp3-HD-Achernar"),
        VoiceCategory::Chirp3Hd
    );
    assert_eq!(category_of("Achird"), VoiceCategory::Uncategorized);
}

#[test]
fn unknown_values_do_not_break_deserialization() {
    let json = r#"{ "voices": [
        { "languageCodes": ["xx-YY"], "name": "xx-YY-Future-A", "ssmlGender": "SOMETHING_NEW", "naturalSampleRateHertz": 16000 }
    ] }"#;

    let voices: Voices = serde_json::from_str(json).unwrap();
    let voice = &voices.as_slice()[0];

    assert_eq!(
        voice.language_codes,
        vec![LanguageCodes::NotClassified("xx-YY".to_string())]
    );
    assert_eq!(voice.gender, SsmlGender::SsmlVoiceGenderUnspecified);
    assert_eq!(
        voice.voice_category,
        VoiceCategory::NotClassified("Future".to_string())
    );
}

#[test]
fn filters_can_be_chained() {
    let voices = sample_voices()
        .filter_by_language_codes(LanguageCodes::Italian)
        .filter_by_gender(SsmlGender::Male);

    assert_eq!(voices.len(), 1);
    assert_eq!(voices.as_slice()[0].name, "it-IT-Neural2-C");
}

#[test]
fn filter_by_voice_categories_matches_any_listed_category() {
    let voices = sample_voices()
        .filter_by_voice_categories(&[VoiceCategory::Standard, VoiceCategory::Chirp3Hd]);

    assert_eq!(voices.len(), 2);
}

#[test]
fn filter_by_text_is_case_insensitive() {
    let voices = sample_voices();

    assert_eq!(voices.filter_by_text("CHIRP3").len(), 1);
    assert_eq!(voices.filter_by_text("chirp3").len(), 1);
}

#[test]
fn get_voice_by_name_is_case_insensitive() {
    assert!(
        sample_voices()
            .get_voice_by_name("IT-it-standard-a")
            .is_some()
    );
    assert!(sample_voices().get_voice_by_name("missing").is_none());
}

#[test]
fn empty_result_does_not_panic() {
    let empty = sample_voices().filter_by_voice_category(VoiceCategory::Studio);

    assert!(empty.is_empty());
    assert!(empty.random_voice().is_none());
    assert!(empty.filter_by_gender(SsmlGender::Female).is_empty());
}

#[test]
fn distinct_categories_and_language_codes() {
    let voices = sample_voices();

    assert_eq!(voices.get_categories().len(), 4);
    assert_eq!(
        voices.get_language_codes(),
        vec![LanguageCodes::Italian, LanguageCodes::EnglishUnitedStates]
    );
}
