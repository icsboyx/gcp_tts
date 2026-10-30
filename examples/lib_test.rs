use std::{fs, hint::black_box, time::Instant};

use gcp_tts::{
    GcpTTS,
    audio_config::AudioConfig,
    speech_config,
    voices::VoiceCategory::{self},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _env_file = dotenv::dotenv().expect("Unable to load '.env' file");
    let key = std::env::var("API_KEY").expect("API_KEY is missing");
    let gcp_tts = GcpTTS::new(key);

    let voices = gcp_tts.get_voices().await?;
    println!("voices listed successfully");
    println!("total voices: {}", voices.len());
    println!("categories: {:?}", voices.get_categories());

    let speech_config = speech_config::SpeechConfig::new(
        voices
            .filter_by_voice_category(VoiceCategory::Standard)
            .random_voice()
            .expect("No voice found in Standard category")
            .clone(),
        AudioConfig::default(),
    );
    let ddd = gcp_tts
        .synthesize_speech("Ciao a tutti, Benvenuti!", speech_config)
        .await?;

    println!(
        "synthesized speech successfully, audio data length: {}",
        ddd.len()
    );

    fs::write(
        format!(
            "{}-output.mp3",
            chrono::Local::now().format("%Y%m%d_%H-%M-%S")
        ),
        ddd,
    )
    .expect("failed to write audio data to file");

    // Benchmarks. Use wisely: any measured operation that calls the GCP API sends
    // one real request per iteration and counts against your quota.
    // Do NOT wrap synthesize_speech in measure/measure_async: synthesis is billed per character.
    measure("filter_by_voice_category", 1000, || {
        voices.filter_by_voice_category(VoiceCategory::Uncategorized)
    });

    measure("filter_by_voice_categories", 1000, || {
        voices.filter_by_voice_categories(&[VoiceCategory::Uncategorized])
    });

    // Network call: measures a full get_voices round trip, reusing the same HTTP client
    measure_async("get_voices", 10, async || gcp_tts.get_voices().await).await;

    Ok(())
}

pub fn measure<T>(name: &str, iterations: u32, mut operation: impl FnMut() -> T) {
    for _ in 0..(iterations / 100) {
        black_box(operation()); // warm up 
    }

    let start = Instant::now();
    for _ in 0..iterations {
        black_box(operation());
    }

    println!("{name}: {:?} per method call", start.elapsed() / iterations);
}

pub async fn measure_async<T>(name: &str, iterations: u32, mut operation: impl AsyncFnMut() -> T) {
    // Warm up
    for _ in 0..(iterations / 100) {
        black_box(operation().await);
    }

    let start = Instant::now();

    for _ in 0..iterations {
        black_box(operation().await);
    }

    println!("{name}: {:?} per method call", start.elapsed() / iterations);
}
