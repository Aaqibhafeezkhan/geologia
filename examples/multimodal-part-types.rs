// Copyright 2026 Andre Cipriani Bandarra
// SPDX-License-Identifier: Apache-2.0

use std::error::Error;

use geologia::prelude::{
    AudioTranscription, Part, PartData, SpeechMetadata, VideoMetadata, WordInfo,
};
use serde_json::json;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    // 1. AudioTranscription with WordInfo
    let transcription_part = Part {
        thought: None,
        thought_signature: None,
        part_metadata: None,
        media_resolution: None,
        video_metadata: None,
        speech_metadata: None,
        data: PartData::AudioTranscription(AudioTranscription {
            text: Some("Welcome to the Geologia Rust client demonstration.".to_string()),
            speaker_label: Some("spk_1".to_string()),
            words: Some(vec![
                WordInfo {
                    word: Some("Welcome".to_string()),
                    start_offset: Some("0.000s".to_string()),
                    end_offset: Some("0.450s".to_string()),
                },
                WordInfo {
                    word: Some("to".to_string()),
                    start_offset: Some("0.460s".to_string()),
                    end_offset: Some("0.550s".to_string()),
                },
                WordInfo {
                    word: Some("Geologia".to_string()),
                    start_offset: Some("0.560s".to_string()),
                    end_offset: Some("1.200s".to_string()),
                },
            ]),
        }),
    };

    let serialized = serde_json::to_string_pretty(&transcription_part)?;
    println!("AudioTranscription JSON:\n{serialized}\n");

    let deserialized: Part = serde_json::from_str(&serialized)?;
    assert_eq!(deserialized, transcription_part);
    println!("Successfully round-tripped AudioTranscription with WordInfo!");

    // 2. VideoMetadata alongside FileData
    let video_part = Part {
        thought: None,
        thought_signature: None,
        part_metadata: None,
        media_resolution: None,
        video_metadata: Some(VideoMetadata {
            start_offset: Some("10s".to_string()),
            end_offset: Some("70s".to_string()),
            fps: Some(1.0),
        }),
        speech_metadata: None,
        data: PartData::FileData(json!({
            "fileUri": "https://generativelanguage.googleapis.com/v1beta/files/video-id-123",
            "mimeType": "video/mp4"
        })),
    };

    let video_serialized = serde_json::to_string_pretty(&video_part)?;
    println!("Video part JSON:\n{video_serialized}\n");
    let video_deserialized: Part = serde_json::from_str(&video_serialized)?;
    assert_eq!(video_deserialized, video_part);
    println!("Successfully round-tripped VideoMetadata on Part!");

    // 3. SpeechMetadata alongside Text
    let speech_part = Part {
        thought: None,
        thought_signature: None,
        part_metadata: None,
        media_resolution: None,
        video_metadata: None,
        speech_metadata: Some(SpeechMetadata {
            speaker: Some("Puck".to_string()),
            style: Some("cheerful and friendly".to_string()),
        }),
        data: PartData::Text("Hello! How can I help you today?".to_string()),
    };

    let speech_serialized = serde_json::to_string_pretty(&speech_part)?;
    println!("Speech part JSON:\n{speech_serialized}\n");
    let speech_deserialized: Part = serde_json::from_str(&speech_serialized)?;
    assert_eq!(speech_deserialized, speech_part);
    println!("Successfully round-tripped SpeechMetadata on Part!");

    Ok(())
}
