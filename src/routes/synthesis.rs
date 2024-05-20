use super::audio_query::AudioQuery;
use crate::{
    aivoice::{Phrase, Style, AIVOICE},
    bridge::{MergedVoiceContainer, VoicePreset, VoicePresetStyle},
    error::{Error, Result},
};

use anyhow::anyhow;
use axum::{extract::Query, Json};
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct AudioQueryQuery {
    pub speaker: u32,
}

pub async fn post_synthesis(
    Query(query): Query<AudioQueryQuery>,
    Json(audio_query): Json<AudioQuery>,
) -> Result<Vec<u8>> {
    let mut pronunciation = vec!["$2_2".to_string()];
    for (i, ap) in audio_query.accent_phrases.iter().enumerate() {
        let mut pronunciation_local = Vec::new();
        for m in &ap.moras {
            pronunciation_local.push(m.text.clone());
        }
        if ap.moras.len() > 1 {
            if ap.accent == 1 {
                pronunciation_local.insert(1, "!".to_string());
                pronunciation_local.insert(0, "^".to_string());
            } else {
                if ap.accent != ap.moras.len() {
                    pronunciation_local.insert(ap.accent, "!".to_string());
                }
                pronunciation_local.insert(1, "^".to_string());
            }
        }
        pronunciation.extend(pronunciation_local);
        if i != audio_query.accent_phrases.len() - 1 {
            if ap.pause_mora.is_some() {
                pronunciation.push("$2_2".to_string());
            } else {
                pronunciation.push("|0".to_string());
            }
        }
    }

    let aivoice = AIVOICE.lock().await;

    info!("Pronunciation: {:?}", pronunciation.join(""));
    let phrase = Phrase::new(pronunciation.join(""));
    aivoice.write_temporary_phrase_dict(Some(&phrase)).await?;

    aivoice.reload_phrase_dictionary().await?;

    let speaker_id = query.speaker / 10;
    let style: Style = num::FromPrimitive::from_u32(query.speaker % 10).ok_or_else(|| {
        Error::SynthesisFailed(anyhow!("Invalid style id: {}", query.speaker % 10))
    })?;

    let speaker = aivoice
        .speakers()
        .values()
        .find(|speaker| *speaker.id() == speaker_id)
        .ok_or_else(|| Error::SpeakerNotFound)?;

    let new_preset = VoicePreset {
        preset_name: "AIVoiceVox".to_string(),
        voice_name: speaker.internal_name().to_string(),
        volume: audio_query.volume_scale as f64,
        speed: audio_query.speed_scale as f64,
        pitch: 2f32.powf(audio_query.pitch_scale) as f64,
        pitch_range: 1.0,
        middle_pause: 750,
        long_pause: 750,
        styles: speaker
            .styles()
            .iter()
            .map(|x| VoicePresetStyle {
                name: x.to_string(),
                value: if *x == style { 1.0 } else { 0.0 },
            })
            .collect(),
        merged_voice_container: MergedVoiceContainer {
            base_pitch_voice_name: speaker.internal_name().to_string(),
            merged_voices: vec![],
        },
    };

    aivoice.set_current_voice_preset_name("AIVoiceVox").await?;
    aivoice.set_voice_preset(&new_preset).await?;

    aivoice
        .set_text(phrase.uuid().hyphenated().to_string().as_str())
        .await?;

    let temp_audio_file = tempfile::Builder::new()
        .suffix(".wav")
        .tempfile()
        .map_err(|e| Error::SynthesisFailed(e.into()))?;
    let temp_audio_file = temp_audio_file.into_temp_path();

    info!("Synthesis started: to {}", temp_audio_file.display());

    aivoice
        .save_audio_to_file(temp_audio_file.to_str().unwrap())
        .await?;

    loop {
        let metadata = tokio::fs::metadata(&temp_audio_file)
            .await
            .map_err(|e| Error::SynthesisFailed(e.into()))?;
        if metadata.len() > 0 {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let file =
        std::fs::File::open(&temp_audio_file).map_err(|e| Error::SynthesisFailed(e.into()))?;

    let (mut header, av_audio) =
        wav_io::read_from_file(file).map_err(|e| Error::SynthesisFailed(anyhow!(e)))?;

    assert_eq!(header.channels, 1);

    let pre_silence = generate_silence(
        header.sample_rate,
        header.channels,
        audio_query.pre_phoneme_length,
    );
    let post_silence = generate_silence(
        header.sample_rate,
        header.channels,
        audio_query.post_phoneme_length,
    );

    let mut audio = pre_silence;
    audio.extend(av_audio);
    audio.extend(post_silence);

    let output_sampling_rate = audio_query
        .output_sampling_rate
        .as_u64()
        .or(audio_query.output_sampling_rate.as_f64().map(|x| x as u64));

    let Some(output_sampling_rate) = output_sampling_rate else {
        return Err(Error::SynthesisFailed(anyhow!(
            "Invalid output sampling rate: {:?}",
            audio_query.output_sampling_rate
        )));
    };
    let Ok(output_sampling_rate) = u32::try_from(output_sampling_rate) else {
        return Err(Error::SynthesisFailed(anyhow!(
            "Invalid output sampling rate: {:?}",
            audio_query.output_sampling_rate
        )));
    };

    let new_audio = wav_io::resample::linear(
        audio,
        header.channels,
        header.sample_rate,
        output_sampling_rate,
    );
    header.sample_rate = output_sampling_rate;

    tokio::fs::remove_file(&temp_audio_file)
        .await
        .map_err(|e| Error::SynthesisFailed(e.into()))?;

    header.sample_rate = output_sampling_rate;
    header.channels = if audio_query.output_stereo { 2 } else { 1 };

    let new_audio = if audio_query.output_stereo {
        wav_io::utils::mono_to_stereo(new_audio)
    } else {
        new_audio
    };

    wav_io::write_to_bytes(&header, &new_audio).map_err(|e| Error::SynthesisFailed(anyhow!(e)))
}

fn generate_silence(sampling_rate: u32, channels: u16, duration: f32) -> Vec<f32> {
    let samples = (sampling_rate as f32 * duration) as usize;
    let silence = vec![0f32; samples * channels as usize];
    silence
}
