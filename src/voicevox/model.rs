use derive_getters::Getters;
use derive_new::new;
use serde::{Deserialize, Serialize};

/* 各フィールドのjsonフィールド名はsnake_caseとする*/

/// モーラ（子音＋母音）ごとの情報。
#[derive(Clone, Debug, new, Getters, Deserialize, Serialize)]
pub struct MoraModel {
    /// 文字。
    pub text: String,
    /// 子音の音素。
    pub consonant: Option<String>,
    /// 子音の音長。
    pub consonant_length: Option<f32>,
    /// 母音の音素。
    pub vowel: String,
    /// 母音の音長。
    pub vowel_length: f32,
    /// 音高。
    pub pitch: f32,
}

/// AccentPhrase (アクセント句ごとの情報)。
#[derive(Clone, Debug, new, Getters, Deserialize, Serialize)]
pub struct AccentPhraseModel {
    /// モーラの配列。
    pub moras: Vec<MoraModel>,
    /// アクセント箇所。
    pub accent: usize,
    /// 後ろに無音を付けるかどうか。
    pub pause_mora: Option<MoraModel>,
    /// 疑問系かどうか。
    #[serde(default)]
    pub is_interrogative: bool,
}

impl AccentPhraseModel {
    pub(super) fn set_pause_mora(&mut self, pause_mora: Option<MoraModel>) {
        self.pause_mora = pause_mora;
    }

    pub(super) fn set_is_interrogative(&mut self, is_interrogative: bool) {
        self.is_interrogative = is_interrogative;
    }
}

/// AudioQuery (音声合成用のクエリ)。
#[allow(clippy::too_many_arguments)]
#[derive(Clone, new, Getters, Deserialize, Serialize)]
pub struct AudioQueryModel {
    /// アクセント句の配列。
    accent_phrases: Vec<AccentPhraseModel>,
    /// 全体の話速。
    speed_scale: f32,
    /// 全体の音高。
    pitch_scale: f32,
    /// 全体の抑揚。
    intonation_scale: f32,
    /// 全体の音量。
    volume_scale: f32,
    /// 音声の前の無音時間。
    pre_phoneme_length: f32,
    /// 音声の後の無音時間。
    post_phoneme_length: f32,
    /// 音声データの出力サンプリングレート。
    output_sampling_rate: u32,
    /// 音声データをステレオ出力するか否か。
    output_stereo: bool,
    /// \[読み取り専用\] AquesTalk風記法。
    ///
    /// [`Synthesizer::audio_query`]が返すもののみ`Some`となる。入力としてのAudioQueryでは無視され
    /// る。
    ///
    /// [`Synthesizer::audio_query`]: crate::Synthesizer::audio_query
    kana: Option<String>,
}
