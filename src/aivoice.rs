use crate::bridge::Host;
pub use crate::bridge::{HostStatus, MergedVoiceContainer, TextEditMode, VoicePreset};
use crate::error::{Error, Result};
use crate::settings_modifier::SettingsModifier;

use derive_getters::Getters;
use fxhash::FxHasher;
use indexmap::IndexMap;
use num_derive::FromPrimitive;
use once_cell::sync::Lazy;
use std::{
    hash::{Hash as _, Hasher as _},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};
use strum::{Display, EnumString};
use tasklist::{kill as taskkill, tasklist};
use tokio::{io::AsyncWriteExt as _, sync::Mutex};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Getters)]
pub struct AiVoice {
    host: Host,
    settings_modifier: Option<SettingsModifier>,
    speakers: IndexMap<String, Speaker>,
}

#[derive(Debug, Clone, Getters)]
pub struct Speaker {
    display_name: String,
    internal_name: String,
    id: u32,
    styles: Vec<Style>,
}

#[derive(Debug, Copy, Clone, EnumString, Display, FromPrimitive, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Style {
    #[strum(serialize = "N")]
    Normal,
    #[strum(serialize = "J")]
    Joy,
    #[strum(serialize = "A")]
    Anger,
    #[strum(serialize = "S")]
    Sorrow,
}

impl Style {
    pub fn to_japanese(self) -> &'static str {
        match self {
            Self::Normal => "ノーマル",
            Self::Joy => "喜び",
            Self::Anger => "怒り",
            Self::Sorrow => "悲しみ",
        }
    }
}

impl Speaker {
    pub fn new(preset: VoicePreset) -> Self {
        let internal_name = preset.voice_name;
        let display_name = preset.preset_name;
        let mut hasher = FxHasher::default();
        internal_name.hash(&mut hasher);
        let id = ((hasher.finish()) % u16::MAX as u64) as u32;
        let mut styles: Vec<_> = preset
            .styles
            .iter()
            .map(|x| Style::from_str(&x.name).unwrap())
            .collect();
        styles.insert(0, Style::Normal);
        Self {
            display_name,
            internal_name,
            id,
            styles,
        }
    }

    pub fn uuid(&self) -> Uuid {
        Uuid::new_v5(&Uuid::NAMESPACE_OID, self.internal_name.as_bytes())
    }
}

#[derive(Debug, Clone, Getters)]
pub struct Phrase {
    uuid: Uuid,
    pronunciation: String,
}

impl Phrase {
    pub fn new(pronunciation: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            pronunciation,
        }
    }
}

impl AiVoice {
    pub fn new() -> Self {
        Self {
            host: Host::new(),
            settings_modifier: None,
            speakers: IndexMap::new(),
        }
    }

    pub fn temporary_phrase_dict_path() -> PathBuf {
        process_path::get_executable_path()
            .unwrap()
            .parent()
            .unwrap()
            .join("temporary_phrase_dict.pdic")
    }

    pub async fn setup(&mut self) -> Result<()> {
        let mut tasks = unsafe { tasklist().into_iter() };
        if let Some((_, aivoice_process_id)) =
            tasks.find(|(task_name, _)| task_name.to_lowercase() == "aivoiceeditor.exe")
        {
            info!("A.I.Voice process id: {}", aivoice_process_id);

            if !unsafe { taskkill(aivoice_process_id) } {
                return Err(Error::TerminateHostFailed);
            }
            info!("Terminated A.I.Voice");
        } else {
            info!("A.I.Voice is not running");
        }

        self.write_temporary_phrase_dict(None).await?;

        let mut settings_modifier = SettingsModifier::new();

        settings_modifier
            .modify(&Self::temporary_phrase_dict_path())
            .await?;
        self.settings_modifier = Some(settings_modifier);

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        self.start_and_connect().await?;

        self.host.set_text_edit_mode(TextEditMode::Text)?;

        let speakers = self.host.speakers()?;

        for speaker in speakers.iter() {
            let preset = self.host.get_voice_preset(speaker)?;
            let speaker = Speaker::new(preset.clone());
            self.speakers.insert(speaker.internal_name.clone(), speaker);
        }

        let voice_preset_names = self.host.voice_preset_names()?;
        if !voice_preset_names.iter().any(|x| x == "AIVoiceVox") {
            self.host.add_voice_preset(VoicePreset {
                preset_name: "AIVoiceVox".to_string(),
                voice_name: self.speakers.values().next().unwrap().internal_name.clone(),
                volume: 1.0,
                speed: 1.0,
                pitch: 1.0,
                pitch_range: 1.0,
                middle_pause: 0,
                long_pause: 0,
                styles: vec![],
                merged_voice_container: MergedVoiceContainer {
                    base_pitch_voice_name: "".to_string(),
                    merged_voices: vec![],
                },
            })?;
        }

        Ok(())
    }

    pub async fn start_and_connect(&self) -> Result<()> {
        info!("Connecting to A.I.Voice...");

        let status = self.host.status();

        match status {
            HostStatus::NotRunning => {
                info!("Starting A.I.Voice...");
                self.host.start()?;
            }
            HostStatus::Idle => {
                info!("A.I.Voice is already running");
                return Ok(());
            }
            _ => {}
        }

        loop {
            info!("Host status: {:?}", self.host.status());

            if self.host.status() == HostStatus::NotConnected {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        info!("Connecting to A.I.Voice...");
        self.host.connect()?;
        info!("Connected to A.I.Voice");

        Ok(())
    }

    pub async fn version(&self) -> Result<String> {
        self.host.reconnect_if_required().await?;
        self.host.version()
    }

    pub async fn write_temporary_phrase_dict(&self, phrase: Option<&Phrase>) -> Result<()> {
        info!(
            "Writing temporary phrase dictionary to {}",
            &Self::temporary_phrase_dict_path().display()
        );
        let mut temporary_phrase_dict =
            tokio::fs::File::create(&Self::temporary_phrase_dict_path())
                .await
                .map_err(Error::WriteDictionaryFailed)?;
        let now = chrono::Local::now();
        let text = format!(
            r#"# ComponentName="AITalk" ComponentVersion="6.0.0.0" UpdateDateTime="{}" Type="Phrase" Version="3.3" Language="Japanese" Count="{}"{}"#,
            now.format("%Y/%m/%d %H:%M:%S.%f"),
            if phrase.is_some() { 1 } else { 0 },
            "\n"
        );
        temporary_phrase_dict
            .write_all(text.as_bytes())
            .await
            .map_err(Error::WriteDictionaryFailed)?;

        if let Some(phrase) = phrase {
            let text = format!(
                r#"num:0{}{}{}$2_2{}$2_2{}"#,
                "\n",
                phrase.uuid.hyphenated(),
                "\n",
                phrase.pronunciation,
                "\n"
            );
            let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode(&text);
            temporary_phrase_dict
                .write_all(&bytes)
                .await
                .map_err(Error::WriteDictionaryFailed)?;
        }
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        if self.host.status() == HostStatus::Idle {
            let mut tasks = unsafe { tasklist().into_iter() };
            if let Some((_, aivoice_process_id)) =
                tasks.find(|(task_name, _)| task_name.to_lowercase() == "aivoiceeditor.exe")
            {
                info!("A.I.Voice process id: {}", aivoice_process_id);

                if !unsafe { taskkill(aivoice_process_id) } {
                    return Err(Error::TerminateHostFailed);
                }
                info!("Terminated A.I.Voice");
            }
        }
        self.settings_modifier
            .as_ref()
            .unwrap()
            .restore_settings()
            .await
            .map_err(|_| Error::TerminateHostFailed)?;

        Ok(())
    }

    pub async fn reload_phrase_dictionary(&self) -> Result<()> {
        self.host.reconnect_if_required().await?;
        self.host.reload_phrase_dictionary()?;

        Ok(())
    }

    pub async fn set_text(&self, text: &str) -> Result<()> {
        self.host.reconnect_if_required().await?;
        self.host.set_text(text)?;

        Ok(())
    }

    pub async fn set_voice_preset(&self, preset: &VoicePreset) -> Result<()> {
        self.host.reconnect_if_required().await?;
        self.host.set_voice_preset(preset)?;

        Ok(())
    }

    pub async fn set_current_voice_preset_name(&self, name: &str) -> Result<()> {
        self.host.reconnect_if_required().await?;
        self.host.set_current_voice_preset_name(name)?;

        Ok(())
    }

    pub async fn save_audio_to_file(&self, path: &str) -> Result<()> {
        self.host.reconnect_if_required().await?;
        self.host.save_audio_to_file(path)?;

        Ok(())
    }
}

pub static AIVOICE: Lazy<Arc<Mutex<AiVoice>>> = Lazy::new(|| Arc::new(Mutex::new(AiVoice::new())));
