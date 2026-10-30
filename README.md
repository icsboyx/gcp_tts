# gcp_tts

Minimal async Rust client for the [Google Cloud Text-to-Speech](https://cloud.google.com/text-to-speech) REST API.

- List all available voices and filter them locally (language, category, gender, text)
- Synthesize text to audio (MP3, OGG Opus, LINEAR16, ...)
- Configure speaking rate, pitch, volume gain, sample rate and effects profiles
- Single shared HTTP client, so connections are reused between requests

## Requirements

- A Google Cloud project with the Text-to-Speech API enabled
- An API key for that project
- A [Tokio](https://tokio.rs) runtime (the API is `async`)

## Installation

The crate is not published on crates.io yet. Add it as a path dependency:

```toml
[dependencies]
gcp_tts = { path = "../gcp_tts" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Usage

```rust
use gcp_tts::{
    GcpTTS, audio_config::AudioConfig, speech_config::SpeechConfig, voices::LanguageCodes,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tts = GcpTTS::new(std::env::var("API_KEY")?);

    // Download the voice list once, then filter it locally.
    let voices = tts.get_voices().await?;
    let voice = voices
        .filter_by_language_codes(LanguageCodes::Italian)
        .random_voice()
        .expect("no Italian voice available")
        .clone();

    // AudioConfig::default() produces MP3.
    let config = SpeechConfig::new(voice, AudioConfig::default());
    let audio = tts.synthesize_speech("Ciao a tutti!", config).await?;
    std::fs::write("output.mp3", audio)?;

    Ok(())
}
```

### Audio settings

```rust
use gcp_tts::audio_config::{AudioConfig, AudioEncoding};

let mut config = AudioConfig::default();
config
    .set_audio_encoding(AudioEncoding::OggOpus)
    .set_speaking_rate(1.2)
    .set_pitch(-2.0);
```

Out-of-range values are clamped to the limits accepted by the API.

## Running the example

Create a `.env` file in the project root:

```
API_KEY=your-gcp-api-key
```

Then run:

```
cargo run --release --example lib_test
```

It lists the voices, synthesizes a short sentence to a timestamped `.mp3` file and runs a few benchmarks.

## Costs and limits

- Synthesis is billed per character by Google Cloud; check the [pricing page](https://cloud.google.com/text-to-speech/pricing).
- A single request accepts at most 5000 bytes of input text.
- Every `get_voices()` call is a network request: call it once and reuse the result.

## License

[MIT](LICENSE)
