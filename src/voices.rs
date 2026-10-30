//! Voice list and local filters.

use crate::{GcpApiError, format_api_error};
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Error, Formatter};

/// Endpoint of the `voices.list` method.
pub static GCP_VOICES_URL: &str = "https://texttospeech.googleapis.com/v1/voices";

/// BCP-47 language code of a voice.
///
/// Codes not listed here are kept as `NotClassified(code)`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum LanguageCodes {
    #[serde(rename = "af-ZA")]
    Afrikaans,
    #[serde(rename = "am-ET")]
    Amharic,
    #[serde(rename = "ar-XA")]
    Arabic,
    #[serde(rename = "bg-BG")]
    Bulgarian,
    #[serde(rename = "bn-IN")]
    Bengali,
    #[serde(rename = "ca-ES")]
    Catalan,
    #[serde(rename = "cmn-CN")]
    MandarinChina,
    #[serde(rename = "cmn-TW")]
    MandarinTaiwan,
    #[serde(rename = "cs-CZ")]
    Czech,
    #[serde(rename = "da-DK")]
    Danish,
    #[serde(rename = "de-DE")]
    German,
    #[serde(rename = "el-GR")]
    Greek,
    #[serde(rename = "en-AU")]
    EnglishAustralia,
    #[serde(rename = "en-GB")]
    EnglishUnitedKingdom,
    #[serde(rename = "en-IN")]
    EnglishIndia,
    #[serde(rename = "en-US")]
    EnglishUnitedStates,
    #[serde(rename = "es-ES")]
    SpanishSpain,
    #[serde(rename = "es-US")]
    SpanishUnitedStates,
    #[serde(rename = "et-EE")]
    Estonian,
    #[serde(rename = "eu-ES")]
    Basque,
    #[serde(rename = "fi-FI")]
    Finnish,
    #[serde(rename = "fil-PH")]
    Filipino,
    #[serde(rename = "fr-CA")]
    FrenchCanada,
    #[serde(rename = "fr-FR")]
    FrenchFrance,
    #[serde(rename = "gl-ES")]
    Galician,
    #[serde(rename = "gu-IN")]
    Gujarati,
    #[serde(rename = "he-IL")]
    Hebrew,
    #[serde(rename = "hi-IN")]
    Hindi,
    #[serde(rename = "hr-HR")]
    Croatian,
    #[serde(rename = "hu-HU")]
    Hungarian,
    #[serde(rename = "id-ID")]
    Indonesian,
    #[serde(rename = "is-IS")]
    Icelandic,
    #[serde(rename = "it-IT")]
    Italian,
    #[serde(rename = "ja-JP")]
    Japanese,
    #[serde(rename = "kn-IN")]
    Kannada,
    #[serde(rename = "ko-KR")]
    Korean,
    #[serde(rename = "lt-LT")]
    Lithuanian,
    #[serde(rename = "lv-LV")]
    Latvian,
    #[serde(rename = "ml-IN")]
    Malayalam,
    #[serde(rename = "mr-IN")]
    Marathi,
    #[serde(rename = "ms-MY")]
    Malay,
    #[serde(rename = "nb-NO")]
    NorwegianBokmal,
    #[serde(rename = "nl-BE")]
    DutchBelgium,
    #[serde(rename = "nl-NL")]
    DutchNetherlands,
    #[serde(rename = "pa-IN")]
    Punjabi,
    #[serde(rename = "pl-PL")]
    Polish,
    #[serde(rename = "pt-BR")]
    PortugueseBrazil,
    #[serde(rename = "pt-PT")]
    PortuguesePortugal,
    #[serde(rename = "ro-RO")]
    Romanian,
    #[serde(rename = "ru-RU")]
    Russian,
    #[serde(rename = "sk-SK")]
    Slovak,
    #[serde(rename = "sl-SI")]
    Slovenian,
    #[serde(rename = "sr-RS")]
    Serbian,
    #[serde(rename = "sv-SE")]
    Swedish,
    #[serde(rename = "sw-KE")]
    Swahili,
    #[serde(rename = "ta-IN")]
    Tamil,
    #[serde(rename = "te-IN")]
    Telugu,
    #[serde(rename = "th-TH")]
    Thai,
    #[serde(rename = "tr-TR")]
    Turkish,
    #[serde(rename = "uk-UA")]
    Ukrainian,
    #[serde(rename = "ur-IN")]
    Urdu,
    #[serde(rename = "vi-VN")]
    Vietnamese,
    #[serde(rename = "yue-HK")]
    CantoneseHongKong,
    #[serde(untagged)]
    NotClassified(String),
}

impl LanguageCodes {
    /// Returns the code as used by the API, e.g. `"it-IT"`.
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Afrikaans => "af-ZA",
            Self::Amharic => "am-ET",
            Self::Arabic => "ar-XA",
            Self::Bulgarian => "bg-BG",
            Self::Bengali => "bn-IN",
            Self::Catalan => "ca-ES",
            Self::MandarinChina => "cmn-CN",
            Self::MandarinTaiwan => "cmn-TW",
            Self::Czech => "cs-CZ",
            Self::Danish => "da-DK",
            Self::German => "de-DE",
            Self::Greek => "el-GR",
            Self::EnglishAustralia => "en-AU",
            Self::EnglishUnitedKingdom => "en-GB",
            Self::EnglishIndia => "en-IN",
            Self::EnglishUnitedStates => "en-US",
            Self::SpanishSpain => "es-ES",
            Self::SpanishUnitedStates => "es-US",
            Self::Estonian => "et-EE",
            Self::Basque => "eu-ES",
            Self::Finnish => "fi-FI",
            Self::Filipino => "fil-PH",
            Self::FrenchCanada => "fr-CA",
            Self::FrenchFrance => "fr-FR",
            Self::Galician => "gl-ES",
            Self::Gujarati => "gu-IN",
            Self::Hebrew => "he-IL",
            Self::Hindi => "hi-IN",
            Self::Croatian => "hr-HR",
            Self::Hungarian => "hu-HU",
            Self::Indonesian => "id-ID",
            Self::Icelandic => "is-IS",
            Self::Italian => "it-IT",
            Self::Japanese => "ja-JP",
            Self::Kannada => "kn-IN",
            Self::Korean => "ko-KR",
            Self::Lithuanian => "lt-LT",
            Self::Latvian => "lv-LV",
            Self::Malayalam => "ml-IN",
            Self::Marathi => "mr-IN",
            Self::Malay => "ms-MY",
            Self::NorwegianBokmal => "nb-NO",
            Self::DutchBelgium => "nl-BE",
            Self::DutchNetherlands => "nl-NL",
            Self::Punjabi => "pa-IN",
            Self::Polish => "pl-PL",
            Self::PortugueseBrazil => "pt-BR",
            Self::PortuguesePortugal => "pt-PT",
            Self::Romanian => "ro-RO",
            Self::Russian => "ru-RU",
            Self::Slovak => "sk-SK",
            Self::Slovenian => "sl-SI",
            Self::Serbian => "sr-RS",
            Self::Swedish => "sv-SE",
            Self::Swahili => "sw-KE",
            Self::Tamil => "ta-IN",
            Self::Telugu => "te-IN",
            Self::Thai => "th-TH",
            Self::Turkish => "tr-TR",
            Self::Ukrainian => "uk-UA",
            Self::Urdu => "ur-IN",
            Self::Vietnamese => "vi-VN",
            Self::CantoneseHongKong => "yue-HK",
            Self::NotClassified(tag) => tag.as_str(),
        }
    }
}

impl Display for LanguageCodes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for LanguageCodes {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Voice model family, derived from the voice name
/// (e.g. `it-IT-Standard-A` → [`VoiceCategory::Standard`]).
///
/// Names without a category part are [`VoiceCategory::Uncategorized`];
/// unknown categories are kept as `NotClassified(name)`.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum VoiceCategory {
    Studio,
    Standard,
    Wavenet,
    Neural2,
    ChirpHd,
    Chirp3Hd,
    News,
    Polyglot,
    Casual,
    Uncategorized,
    #[serde(untagged)]
    NotClassified(String),
}

impl AsRef<str> for VoiceCategory {
    fn as_ref(&self) -> &str {
        match self {
            Self::Studio => "Studio",
            Self::Standard => "Standard",
            Self::Wavenet => "Wavenet",
            Self::Neural2 => "Neural2",
            Self::ChirpHd => "Chirp-HD",
            Self::Chirp3Hd => "Chirp3-HD",
            Self::News => "News",
            Self::Polyglot => "Polyglot",
            Self::Uncategorized => "Uncategorized",
            Self::Casual => "Casual",
            Self::NotClassified(val) => val.as_str(),
        }
    }
}

impl From<&str> for VoiceCategory {
    fn from(value: &str) -> Self {
        match value {
            "Studio" => Self::Studio,
            "Standard" => Self::Standard,
            "Wavenet" => Self::Wavenet,
            "Neural2" => Self::Neural2,
            "Chirp-HD" => Self::ChirpHd,
            "Chirp3-HD" => Self::Chirp3Hd,
            "News" => Self::News,
            "Polyglot" => Self::Polyglot,
            "Uncategorized" => Self::Uncategorized,
            "Casual" => Self::Casual,
            val => Self::NotClassified(val.to_string()),
        }
    }
}

impl From<String> for VoiceCategory {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&String> for VoiceCategory {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawVoice {
    language_codes: Vec<LanguageCodes>,
    name: String,
    #[serde(rename = "ssmlGender")]
    gender: SsmlGender,
    natural_sample_rate_hertz: i64,
}

impl From<RawVoice> for Voice {
    fn from(raw: RawVoice) -> Self {
        let mut striped = raw.name.clone();
        for lc in raw.language_codes.iter() {
            let prefix_len = lc.as_str().len();
            if raw
                .name
                .get(..prefix_len)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(lc.as_str()))
                && raw.name.as_bytes().get(prefix_len) == Some(&b'-')
            {
                striped = raw.name[prefix_len + 1..].to_string();
                break;
            }
        }

        let (voice_category, _voice_name) = striped
            .rsplit_once('-')
            .map(|(category, name)| (category.to_string(), name.to_string()))
            .unwrap_or_else(|| ("Uncategorized".to_string(), striped.to_string()));
        Self {
            language_codes: raw.language_codes,
            name: raw.name,
            gender: raw.gender,
            natural_sample_rate_hertz: raw.natural_sample_rate_hertz,
            voice_category: voice_category.into(),
        }
    }
}

/// A voice returned by the API.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
#[serde(from = "RawVoice")]
pub struct Voice {
    pub language_codes: Vec<LanguageCodes>,
    pub name: String,
    #[serde(rename = "ssmlGender")]
    pub gender: SsmlGender,
    pub natural_sample_rate_hertz: i64,
    #[serde(skip)]
    pub voice_category: VoiceCategory,
}

/// Gender of a voice.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SsmlGender {
    #[serde(rename = "FEMALE")]
    Female,
    #[serde(rename = "MALE")]
    Male,
    #[serde(rename = "NEUTRAL")]
    Neutral,
    #[serde(rename = "SSML_VOICE_GENDER_UNSPECIFIED", other)]
    SsmlVoiceGenderUnspecified,
}

/// Collection of voices with local filters.
///
/// Filters return a new `Voices` and can be chained; they never call the API.
///
/// ```no_run
/// # use gcp_tts::voices::{Voices, SsmlGender, VoiceCategory, LanguageCodes};
/// # fn pick(voices: &Voices) {
/// let voice = voices
///     .filter_by_language_codes(LanguageCodes::Italian)
///     .filter_by_voice_category(VoiceCategory::Neural2)
///     .filter_by_gender(SsmlGender::Female)
///     .random_voice()
///     .cloned();
/// # }
/// ```
#[derive(Deserialize, Clone, Debug, Default, Serialize)]
pub struct Voices {
    voices: Vec<Voice>,
}

impl Voices {
    /// Downloads the voice list. Called by [`GcpTTS::get_voices`](crate::GcpTTS::get_voices).
    pub(crate) async fn get_voices(
        api_key: impl AsRef<str>,
        http_client: &reqwest::Client,
    ) -> Result<Voices, GcpApiError> {
        let url = reqwest::Url::parse(GCP_VOICES_URL)?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-goog-api-key", api_key.as_ref().parse()?);

        let request_debug = format!("GET {GCP_VOICES_URL}");

        let response = http_client.get(url).headers(headers).send().await?;

        if !response.status().is_success() {
            return Err(format_api_error(request_debug, response).await?);
        }

        Ok(response.json::<Voices>().await?)
    }

    /// Borrows the voices as a slice.
    pub fn as_slice(&self) -> &[Voice] {
        self.voices.as_slice()
    }

    // To get owned values
    /// Returns an owned copy of the voices.
    pub fn to_vec(&self) -> Vec<Voice> {
        self.voices.clone()
    }

    /// Builds a collection from an existing list of voices.
    pub fn from_vec(voices: Vec<Voice>) -> Self {
        Self { voices }
    }

    /// Returns `true` if there are no voices.
    pub fn is_empty(&self) -> bool {
        self.voices.is_empty()
    }
    /// Returns the distinct voice categories, in first-seen order.
    pub fn get_categories(&self) -> Vec<VoiceCategory> {
        let mut unique: Vec<VoiceCategory> = Vec::new();

        for voice in &self.voices {
            let category = &voice.voice_category;
            if !unique.contains(category) {
                unique.push(category.clone());
            }
        }

        unique
    }

    /// Returns the distinct language codes, in first-seen order.
    pub fn get_language_codes(&self) -> Vec<LanguageCodes> {
        let mut unique: Vec<LanguageCodes> = Vec::new();

        for voice in &self.voices {
            for language_code in &voice.language_codes {
                if !unique.contains(language_code) {
                    unique.push(language_code.clone());
                }
            }
        }

        unique
    }

    /// Finds a voice by its full name (case-insensitive), e.g. `"it-IT-Standard-A"`.
    pub fn get_voice_by_name(&self, name: impl AsRef<str>) -> Option<&Voice> {
        self.voices
            .iter()
            .find(|voice| voice.name.eq_ignore_ascii_case(name.as_ref()))
    }

    /// Keeps only voices of the given category.
    pub fn filter_by_voice_category(&self, voice_category: VoiceCategory) -> Self {
        let voices = self
            .voices
            .iter()
            .filter(|v| v.voice_category == voice_category)
            .cloned()
            .collect::<Vec<_>>();
        Self { voices }
    }

    /// Keeps only voices whose category is in `voice_categories`.
    pub fn filter_by_voice_categories(&self, voice_categories: &[VoiceCategory]) -> Self {
        let voices = self
            .voices
            .iter()
            .filter(|v| voice_categories.contains(&v.voice_category))
            .cloned()
            .collect();
        Self { voices }
    }

    /// Keeps only voices of the given gender.
    pub fn filter_by_gender(&self, gender: SsmlGender) -> Self {
        let filtered_voices = self
            .voices
            .iter()
            .filter(|voice| voice.gender == gender)
            .cloned()
            .collect::<Vec<Voice>>();

        Self {
            voices: filtered_voices,
        }
    }

    /// Keeps only voices that support the given language.
    pub fn filter_by_language_codes(&self, language_code: LanguageCodes) -> Self {
        let filtered_voices = self
            .voices
            .iter()
            .filter(|voice| {
                voice
                    .language_codes
                    .iter()
                    .any(|lc| lc.as_str().eq_ignore_ascii_case(language_code.as_str()))
            })
            .cloned()
            .collect::<Vec<Voice>>();

        Self {
            voices: filtered_voices,
        }
    }

    /// Keeps only voices whose fields contain `contained_text` (case-insensitive).
    pub fn filter_by_text(&self, contained_text: impl AsRef<str>) -> Self {
        let contained_text = contained_text.as_ref().to_lowercase();
        let filtered_voices = self
            .voices
            .iter()
            .filter(|voice| {
                let dbg_text = &format!("{:?}", voice).to_lowercase();
                dbg_text.contains(&contained_text)
            })
            .cloned()
            .collect::<Vec<Voice>>();

        Self {
            voices: filtered_voices,
        }
    }

    /// Returns a random voice, or `None` if the collection is empty.
    pub fn random_voice(&self) -> Option<&Voice> {
        let mut rng = rand::rng();
        self.voices.choose(&mut rng)
    }

    /// Returns the number of voices.
    pub fn len(&self) -> usize {
        self.voices.len()
    }
}
