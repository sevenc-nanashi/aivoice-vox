use crate::error::{Error, Result};
use crate::voicevox::model::AccentPhraseModel;
use crate::voicevox::open_jtalk::OpenJtalk;

use axum::{extract::Query, Json};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
pub struct AudioQueryParams {
    text: String,
    speaker: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioQuery {
    #[serde(rename = "accent_phrases")]
    pub accent_phrases: Vec<AccentPhraseModel>,
    pub speed_scale: f32,
    pub pitch_scale: f32,
    pub intonation_scale: f32,
    pub volume_scale: f32,
    pub pre_phoneme_length: f32,
    pub post_phoneme_length: f32,
    pub output_sampling_rate: usize,
    pub output_stereo: bool,
    pub kana: String,
}

pub static OPEN_JTALK: Lazy<Arc<Mutex<OpenJtalk>>> = Lazy::new(|| {
    let path = process_path::get_executable_path()
        .unwrap()
        .join("open_jtalk_dic_utf_8-1.11");
    let open_jtalk = OpenJtalk::new_with_initialize(if cfg!(debug_assertions) {
        "./open_jtalk_dic_utf_8-1.11"
    } else {
        path.to_str().unwrap()
    })
    .unwrap();
    Arc::new(Mutex::new(open_jtalk))
});

pub async fn post_audio_query(Query(query): Query<AudioQueryParams>) -> Result<Json<AudioQuery>> {
    let open_jtalk = OPEN_JTALK.lock().await;

    let accent_phrases = open_jtalk
        .create_accent_phrases(&query.text[..])
        .await
        .map_err(|e| Error::AnalyzeFailed(e.into()))?;

    Ok(Json(AudioQuery {
        accent_phrases,
        speed_scale: 1.0,
        pitch_scale: 1.0,
        intonation_scale: 1.0,
        volume_scale: 1.0,
        pre_phoneme_length: 0.0,
        post_phoneme_length: 0.0,
        output_sampling_rate: 24000,
        output_stereo: true,
        kana: query.text.clone(),
    }))
}

pub async fn post_accent_phrases(
    Query(query): Query<AudioQueryParams>,
) -> Result<Json<Vec<AccentPhraseModel>>> {
    let open_jtalk = OPEN_JTALK.lock().await;

    let accent_phrases = open_jtalk
        .create_accent_phrases(&query.text[..])
        .await
        .map_err(|e| Error::AnalyzeFailed(e.into()))?;

    Ok(Json(accent_phrases))
}
