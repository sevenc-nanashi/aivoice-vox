use crate::error::{Error, Result};

use indoc::indoc;
use std::env;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Debug)]
pub struct SettingsModifier {}

impl SettingsModifier {
    pub fn new() -> Self {
        Self {}
    }

    pub fn aivoice_setting_dir() -> PathBuf {
        Path::new(&env::var("LOCALAPPDATA").unwrap())
            .join("AI")
            .join("A.I.VOICE Editor")
            .join("1.0")
    }
    pub fn aivoice_setting_path() -> PathBuf {
        SettingsModifier::aivoice_setting_dir().join("Standard.settings")
    }
    pub fn aivoice_backup_setting_path() -> PathBuf {
        SettingsModifier::aivoice_setting_dir().join("Standard.settings.bak")
    }

    pub async fn modify(&mut self, temporary_phrase_dict_path: &Path) -> Result<()> {
        info!(
            "A.I.Voice setting path: {}",
            &SettingsModifier::aivoice_setting_path().display()
        );

        let mut settings = tokio::fs::read_to_string(&SettingsModifier::aivoice_setting_path())
            .await
            .map_err(|e| Error::SettingsParseFailed(e.into()))?;

        let modify_setting = |settings: &str, key: &str, value: &str| -> String {
            let key_start = settings
                .find(format!("<{}>", key).as_str())
                .unwrap_or_else(|| panic!("key_start not found, key: {}", key))
                + key.len()
                + 2;
            let key_end = settings[key_start..]
                .find(format!("</{}>", key).as_str())
                .unwrap_or_else(|| panic!("key_end not found, key: {}", key))
                + key_start;

            format!(
                "{}{}{}",
                &settings[..key_start],
                value,
                &settings[key_end..]
            )
        };

        let new_phrase_dic = format!(
            indoc! {
            r#"
              <FilePath>
                <IsSpecialFolderEnabled>false</IsSpecialFolderEnabled>
                <SpecialFolder>Personal</SpecialFolder>
                <PartialPath>{}</PartialPath>
              </FilePath>
              <IsEnabled>true</IsEnabled>
            "#
            },
            html_escape::encode_text(&temporary_phrase_dict_path.display().to_string())
        );
        settings = modify_setting(&settings, "PhraseDic", &new_phrase_dic);
        settings = modify_setting(&settings, "BeginPause", "0");
        settings = modify_setting(&settings, "TermPause", "0");
        settings = modify_setting(&settings, "BitDepth", "0");
        settings = modify_setting(&settings, "SamplesPerSec", "48000");
        settings = modify_setting(&settings, "PcmAudioType", "Linear");
        settings = modify_setting(&settings, "FilePathSelectionMode", "FileSaveDialog");
        settings = modify_setting(&settings, "IsTextFileCreated", "false");
        settings = modify_setting(&settings, "SplitCondition", "None");

        if tokio::fs::metadata(&SettingsModifier::aivoice_backup_setting_path())
            .await
            .is_ok()
        {
            warn!("A.I.Voice settings backup file already exists, skipping backup (perhaps the previous run was interrupted)");
        } else {
            tokio::fs::copy(
                &SettingsModifier::aivoice_setting_path(),
                &SettingsModifier::aivoice_backup_setting_path(),
            )
            .await
            .map_err(|e| Error::SettingsParseFailed(e.into()))?;
        }

        tokio::fs::write(&SettingsModifier::aivoice_setting_path(), settings)
            .await
            .map_err(|e| Error::SettingsParseFailed(e.into()))?;

        Ok(())
    }

    pub async fn restore_settings(&self) -> std::result::Result<(), std::io::Error> {
        info!("Restoring A.I.Voice settings...");
        tokio::fs::copy(
            SettingsModifier::aivoice_backup_setting_path(),
            SettingsModifier::aivoice_setting_path(),
        )
        .await?;
        tokio::fs::remove_file(SettingsModifier::aivoice_backup_setting_path()).await?;
        info!("A.I.Voice settings restored");
        Ok(())
    }
}
