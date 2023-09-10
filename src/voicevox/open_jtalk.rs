use super::full_context_label::Utterance;
use super::model::*;
use super::user_dict::UserDict;

use std::io::Write;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use tempfile::NamedTempFile;

use ::open_jtalk::*;

#[derive(thiserror::Error, Debug)]
pub enum OpenJtalkError {
    #[error("open_jtalk load error")]
    Load { mecab_dict_dir: PathBuf },
    #[error("open_jtalk extract_fullcontext error")]
    ExtractFullContext {
        text: String,
        #[source]
        source: Option<anyhow::Error>,
    },
    #[error("open_jtalk use_user_dict error")]
    UseUserDict(String),
}

type Result<T> = std::result::Result<T, OpenJtalkError>;

/// テキスト解析器としてのOpen JTalk。
pub struct OpenJtalk {
    resources: Mutex<Resources>,
    dict_dir: Option<PathBuf>,
}

struct Resources {
    mecab: ManagedResource<Mecab>,
    njd: ManagedResource<Njd>,
    jpcommon: ManagedResource<JpCommon>,
}

#[allow(unsafe_code)]
unsafe impl Send for Resources {}

impl OpenJtalk {
    pub fn new_without_dic() -> Self {
        Self {
            resources: Mutex::new(Resources {
                mecab: ManagedResource::initialize(),
                njd: ManagedResource::initialize(),
                jpcommon: ManagedResource::initialize(),
            }),
            dict_dir: None,
        }
    }
    pub fn new_with_initialize(open_jtalk_dict_dir: impl AsRef<Path>) -> Result<Self> {
        let mut s = Self::new_without_dic();
        s.load(open_jtalk_dict_dir)?;
        Ok(s)
    }

    // 先に`load`を呼ぶ必要がある。
    /// ユーザー辞書を設定する。
    ///
    /// この関数を呼び出した後にユーザー辞書を変更した場合は、再度この関数を呼ぶ必要がある。
    pub fn use_user_dict(&self, user_dict: &UserDict) -> Result<()> {
        let dict_dir = self
            .dict_dir
            .as_ref()
            .and_then(|dict_dir| dict_dir.to_str())
            .ok_or_else(|| OpenJtalkError::UseUserDict("辞書が読み込まれていません".to_string()))?;

        // ユーザー辞書用のcsvを作成
        let mut temp_csv =
            NamedTempFile::new().map_err(|e| OpenJtalkError::UseUserDict(e.to_string()))?;
        temp_csv
            .write_all(user_dict.to_mecab_format().as_bytes())
            .map_err(|e| OpenJtalkError::UseUserDict(e.to_string()))?;
        let temp_csv_path = temp_csv.into_temp_path();
        let temp_dict =
            NamedTempFile::new().map_err(|e| OpenJtalkError::UseUserDict(e.to_string()))?;
        let temp_dict_path = temp_dict.into_temp_path();

        // Mecabでユーザー辞書をコンパイル
        // TODO: エラー（SEGV）が出るパターンを把握し、それをRust側で防ぐ。
        mecab_dict_index(&[
            "mecab-dict-index",
            "-d",
            dict_dir,
            "-u",
            temp_dict_path.to_str().unwrap(),
            "-f",
            "utf-8",
            "-t",
            "utf-8",
            temp_csv_path.to_str().unwrap(),
            "-q",
        ]);

        let Resources { mecab, .. } = &mut *self.resources.lock().unwrap();

        let result = mecab.load_with_userdic(Path::new(dict_dir), Some(Path::new(&temp_dict_path)));

        if !result {
            return Err(OpenJtalkError::UseUserDict(
                "辞書のコンパイルに失敗しました".to_string(),
            ));
        }

        Ok(())
    }

    pub(crate) fn extract_fullcontext(&self, text: impl AsRef<str>) -> Result<Vec<String>> {
        let Resources {
            mecab,
            njd,
            jpcommon,
        } = &mut *self.resources.lock().unwrap();

        jpcommon.refresh();
        njd.refresh();
        mecab.refresh();

        let mecab_text =
            text2mecab(text.as_ref()).map_err(|e| OpenJtalkError::ExtractFullContext {
                text: text.as_ref().into(),
                source: Some(e.into()),
            })?;
        if mecab.analysis(mecab_text) {
            njd.mecab2njd(
                mecab
                    .get_feature()
                    .ok_or(OpenJtalkError::ExtractFullContext {
                        text: text.as_ref().into(),
                        source: None,
                    })?,
                mecab.get_size(),
            );
            njd.set_pronunciation();
            njd.set_digit();
            njd.set_accent_phrase();
            njd.set_accent_type();
            njd.set_unvoiced_vowel();
            njd.set_long_vowel();
            jpcommon.njd2jpcommon(njd);
            jpcommon.make_label();
            jpcommon
                .get_label_feature_to_iter()
                .ok_or_else(|| OpenJtalkError::ExtractFullContext {
                    text: text.as_ref().into(),
                    source: None,
                })
                .map(|iter| iter.map(|s| s.to_string()).collect())
        } else {
            Err(OpenJtalkError::ExtractFullContext {
                text: text.as_ref().into(),
                source: None,
            })
        }
    }

    fn load(&mut self, open_jtalk_dict_dir: impl AsRef<Path>) -> Result<()> {
        let result = self
            .resources
            .lock()
            .unwrap()
            .mecab
            .load(open_jtalk_dict_dir.as_ref());
        if result {
            self.dict_dir = Some(open_jtalk_dict_dir.as_ref().into());
            Ok(())
        } else {
            self.dict_dir = None;
            Err(OpenJtalkError::Load {
                mecab_dict_dir: open_jtalk_dict_dir.as_ref().into(),
            })
        }
    }

    pub fn dict_loaded(&self) -> bool {
        self.dict_dir.is_some()
    }

    pub async fn create_accent_phrases(
        &self,
        text: &str,
    ) -> super::full_context_label::Result<Vec<AccentPhraseModel>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let utterance = Utterance::extract_full_context_label(self, text)?;

        let accent_phrases: Vec<AccentPhraseModel> = utterance
            .breath_groups()
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut accum_vec, (i, breath_group)| {
                accum_vec.extend(breath_group.accent_phrases().iter().enumerate().map(
                    |(j, accent_phrase)| {
                        let moras = accent_phrase
                            .moras()
                            .iter()
                            .map(|mora| {
                                let mora_text = mora
                                    .phonemes()
                                    .iter()
                                    .map(|phoneme| phoneme.phoneme().to_string())
                                    .collect::<Vec<_>>()
                                    .join("");

                                let (consonant, consonant_length) =
                                    if let Some(consonant) = mora.consonant() {
                                        (Some(consonant.phoneme().to_string()), Some(0.))
                                    } else {
                                        (None, None)
                                    };

                                MoraModel::new(
                                    mora_to_text(mora_text),
                                    consonant,
                                    consonant_length,
                                    mora.vowel().phoneme().into(),
                                    0.,
                                    0.,
                                )
                            })
                            .collect();

                        let pause_mora = if i != utterance.breath_groups().len() - 1
                            && j == breath_group.accent_phrases().len() - 1
                        {
                            Some(MoraModel::new(
                                "、".into(),
                                None,
                                None,
                                "pau".into(),
                                0.,
                                0.,
                            ))
                        } else {
                            None
                        };

                        AccentPhraseModel::new(
                            moras,
                            *accent_phrase.accent(),
                            pause_mora,
                            *accent_phrase.is_interrogative(),
                        )
                    },
                ));

                accum_vec
            });
        Ok(accent_phrases)
    }
}

fn mora_to_text(mora: impl AsRef<str>) -> String {
    let last_char = mora.as_ref().chars().last().unwrap();
    let mora = if ['A', 'I', 'U', 'E', 'O'].contains(&last_char) {
        format!(
            "{}{}",
            &mora.as_ref()[0..mora.as_ref().len() - 1],
            last_char.to_lowercase()
        )
    } else {
        mora.as_ref().to_string()
    };
    // もしカタカナに変換できなければ、引数で与えた文字列がそのまま返ってくる
    super::mora_list::mora2text(&mora).to_string()
}
