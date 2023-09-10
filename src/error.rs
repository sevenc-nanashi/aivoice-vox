use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code, clippy::enum_variant_names)]
pub enum Error {
    #[error("A.I.Voiceの初期化に失敗しました")]
    InitializeFailed,
    #[error("A.I.Voiceの起動に失敗しました")]
    StartHostFailed,
    #[error("A.I.Voiceへの接続に失敗しました")]
    ConnectFailed,
    #[error("A.I.Voiceのバージョン取得に失敗しました")]
    VersionFailed,
    #[error("A.I.Voiceのステータス取得に失敗しました")]
    StatusFailed,
    #[error("A.I.Voiceのスピーカー取得に失敗しました")]
    SpeakersFailed,
    #[error("A.I.Voiceのプロセスを見つけられませんでした")]
    ProcessNotFound,
    #[error("A.I.VoiceのAPI呼び出しに失敗しました：{0}")]
    ApiFailed(String),
    #[error("A.I.Voiceの終了に失敗しました")]
    TerminateHostFailed,
    #[error("設定をパースできませんでした")]
    SettingsParseFailed(#[source] anyhow::Error),
    #[error("辞書を書き込めませんでした")]
    WriteDictionaryFailed(#[source] tokio::io::Error),
    #[error("画像を読み込めませんでした")]
    ReadImageFailed(#[source] anyhow::Error),
    #[error("辞書を読み込めませんでした")]
    ReadDictionaryFailed(#[source] anyhow::Error),
    #[error("辞書の操作に失敗しました")]
    DictionaryOperationFailed(#[source] anyhow::Error),
    #[error("解析中にエラーが発生しました")]
    AnalyzeFailed(#[source] anyhow::Error),
    #[error("音声合成中にエラーが発生しました")]
    SynthesisFailed(#[source] anyhow::Error),
    #[error("話者が見つかりませんでした")]
    SpeakerNotFound,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        Json(&ErrorResponse {
            error: self.to_string(),
        })
        .into_response()
    }
}
