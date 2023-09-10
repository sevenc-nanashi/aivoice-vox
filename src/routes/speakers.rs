use crate::aivoice::Style;
use crate::{
    aivoice::{Speaker, AIVOICE},
    error::{ErrorResponse, Result},
    icon_manager::ICON_MANAGER,
};

use axum::{http::StatusCode, response::IntoResponse, Json};
use base64::Engine as _;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VvSpeaker {
    pub name: String,
    pub speaker_uuid: String,
    pub styles: Vec<VvStyle>,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VvSpeakerInfo {
    pub policy: String,
    pub portrait: String,
    pub style_infos: Vec<VvStyleInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VvStyleInfo {
    pub id: u32,
    pub icon: String,
    pub portrait: String,
    pub voice_samples: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupportedFeatures {
    pub permitted_synthesis_morphing: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VvStyle {
    pub name: String,
    pub id: u32,
}

pub async fn get_speakers() -> Result<Json<Vec<VvSpeaker>>> {
    let aivoice = AIVOICE.lock().await;
    let version = aivoice.version().await?;
    Ok(Json(
        aivoice
            .speakers()
            .values()
            .map(|speaker| {
                Ok(VvSpeaker {
                    name: speaker.display_name().to_string(),
                    speaker_uuid: speaker.uuid().hyphenated().to_string(),
                    version: version.clone(),
                    styles: speaker
                        .styles()
                        .iter()
                        .map(|style| VvStyle {
                            name: style.to_japanese().to_string(),
                            id: speaker.id() * 10 + *style as u32,
                        })
                        .collect(),
                })
            })
            .collect::<Result<Vec<_>>>()?,
    ))
}

#[derive(Debug, Deserialize)]
pub struct SpeakerInfoQuery {
    pub speaker_uuid: String,
}

pub async fn get_speaker_info(query: axum::extract::Query<SpeakerInfoQuery>) -> impl IntoResponse {
    let aivoice = AIVOICE.lock().await;
    let icon = ICON_MANAGER.lock().await;

    let speaker: &Speaker = match aivoice.speakers().values().find(|speaker| {
        speaker
            .uuid()
            .hyphenated()
            .to_string()
            .eq_ignore_ascii_case(&query.speaker_uuid)
    }) {
        Some(speaker) => speaker,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Speaker not found".into(),
                })
                .into_response(),
            )
                .into_response()
        }
    };

    let portraits = icon.portraits().get(speaker.internal_name()).unwrap();
    let icons = icon.icons().get(speaker.internal_name()).unwrap();

    Json(VvSpeakerInfo {
        policy: "[A.I.Voiceのキャラクター規約](https://aivoice.jp/character/)を参照してください。"
            .into(),
        portrait: base64::engine::general_purpose::STANDARD_NO_PAD.encode(portraits.normal()),
        style_infos: speaker
            .styles()
            .iter()
            .map(|style| VvStyleInfo {
                id: speaker.id() * 10 + *style as u32,
                icon: base64::engine::general_purpose::STANDARD_NO_PAD.encode(match style {
                    Style::Normal => icons.normal(),
                    Style::Joy => icons.joy(),
                    Style::Anger => icons.anger(),
                    Style::Sorrow => icons.sorrow(),
                }),
                portrait: base64::engine::general_purpose::STANDARD_NO_PAD.encode(match style {
                    Style::Normal => portraits.normal(),
                    Style::Joy => portraits.joy(),
                    Style::Anger => portraits.anger(),
                    Style::Sorrow => portraits.sorrow(),
                }),
                voice_samples: vec![],
            })
            .collect(),
    })
    .into_response()
}

pub async fn get_is_initialized_speaker() -> &'static str {
    "true"
}
