// Example: Generate music using ACE-Step
//
// Run with: cargo run --example generate_music
//
// Prerequisites: ACE-Step server must be running on port 7865
// Start with: acestep --port 7865

use engine_ai_music::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("ACE-Step Music Generation Example");
    println!("==================================\n");

    // Create client
    let client = AceStepClient::new();

    // Check health
    println!("Checking ACE-Step service...");
    match client.health_check().await {
        Ok(health) => {
            println!("✓ Service is running");
            println!("  Status: {}", health.status);
            println!("  Model loaded: {}", health.model_loaded);
            println!("  Device: {}\n", health.device);
        }
        Err(e) => {
            eprintln!("✗ Service not available: {}", e);
            eprintln!("\nMake sure ACE-Step is running:");
            eprintln!("  acestep --port 7865");
            return Ok(());
        }
    }

    // Generate some music
    let examples = vec![
        (
            "Epic orchestral battle music with drums and brass",
            MusicDuration::Short,
            MusicStyle::Cinematic,
            "battle_music.wav",
        ),
        (
            "Calm ambient meditation music with soft pads",
            MusicDuration::Medium,
            MusicStyle::Ambient,
            "ambient_music.wav",
        ),
        (
            "Upbeat electronic dance music with heavy bass",
            MusicDuration::Short,
            MusicStyle::Electronic,
            "edm_music.wav",
        ),
    ];

    for (i, (prompt, duration, style, filename)) in examples.iter().enumerate() {
        println!("\n[{}/{}] Generating: {}", i + 1, examples.len(), prompt);

        let request = MusicGenerationRequest::new(*prompt)
            .with_duration(*duration)
            .with_style(style.clone())
            .instrumental()
            .with_steps(20); // Lower steps for faster generation

        match client.generate_and_save(request, filename).await {
            Ok(_) => {
                println!("✓ Saved to: {}", filename);
            }
            Err(e) => {
                eprintln!("✗ Failed to generate: {}", e);
            }
        }
    }

    println!("\n\nDone! Generated music files are ready.");
    println!("You can play them with any audio player.");

    Ok(())
}
