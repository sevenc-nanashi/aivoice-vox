use crate::error::{Error, Result};

use serde::{Deserialize, Serialize};
use std::ffi::c_char;
use tasklist::{kill, tasklist};
use tracing::{info, warn};

#[link(name = "bridge", kind = "static")]
#[allow(dead_code)]
extern "C" {
    fn bridge_com_initialize() -> bool;
    fn bridge_initialize_with_hostname() -> *const c_char;
    fn bridge_get_status() -> i32;
    fn bridge_start_host() -> bool;
    fn bridge_terminate_host() -> bool;
    fn bridge_connect() -> bool;
    fn bridge_get_version() -> *const c_char;
    fn bridge_set_text_edit_mode(mode: i32) -> bool;

    fn bridge_get_speakers() -> *const *const c_char;
    fn bridge_get_voice_preset_names() -> *const *const c_char;

    fn bridge_add_voice_preset(json: *const c_char) -> bool;
    fn bridge_get_voice_preset(preset_name: *const c_char) -> *const c_char;
    fn bridge_set_voice_preset(json: *const c_char) -> bool;

    fn bridge_reload_phrase_dictionary() -> bool;

    fn bridge_set_text(text: *const c_char) -> bool;
    fn bridge_set_current_voice_preset_name(preset_name: *const c_char) -> bool;

    fn bridge_save_audio_to_file(path: *const c_char) -> bool;

    fn bridge_free(ptr: *mut c_char);
    fn bridge_free_array(ptr: *mut *const c_char);
}

#[derive(Debug)]
pub struct Host {
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub enum HostStatus {
    Error,
    NotRunning,
    NotConnected,
    Idle,
    Busy,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum TextEditMode {
    Text = 0,
    Line = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VoicePreset {
    pub preset_name: String,
    pub voice_name: String,
    pub volume: f64,
    pub speed: f64,
    pub pitch: f64,
    pub pitch_range: f64,
    pub middle_pause: i64,
    pub long_pause: i64,
    pub styles: Vec<VoicePresetStyle>,
    pub merged_voice_container: MergedVoiceContainer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VoicePresetStyle {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MergedVoiceContainer {
    pub base_pitch_voice_name: String,
    pub merged_voices: Vec<MergedVoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MergedVoice {
    pub voice_name: String,
}

fn ptr_to_array(ptr: *const *const c_char) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();
    let mut i = 0;
    unsafe {
        loop {
            let ptr = *ptr.offset(i);
            if ptr.is_null() {
                break;
            }
            let c_str = std::ffi::CStr::from_ptr(ptr);
            let (speaker_name, _, invalid) = encoding_rs::SHIFT_JIS.decode(c_str.to_bytes());
            if invalid {
                warn!("Invalid SHIFT-JIS sequence: {}", speaker_name);
                continue;
            }
            ret.push(speaker_name.to_string());
            i += 1;
        }
    }
    ret
}

impl Host {
    pub fn new() -> Self {
        Self::initialize().unwrap();
        let name = Self::hostname();
        info!("Hostname: {}", name);
        Self { name }
    }

    pub fn status(&self) -> HostStatus {
        unsafe {
            let status = bridge_get_status();
            match status {
                -1 => HostStatus::Error,
                0 => HostStatus::NotRunning,
                1 => HostStatus::NotConnected,
                2 => HostStatus::Idle,
                3 => HostStatus::Busy,
                _ => panic!("Invalid status: {}", status),
            }
        }
    }

    pub fn start(&self) -> Result<()> {
        unsafe {
            let success = bridge_start_host();

            if success {
                Ok(())
            } else {
                Err(Error::StartHostFailed)
            }
        }
    }

    pub fn connect(&self) -> Result<()> {
        unsafe {
            let success = bridge_connect();
            if success {
                Ok(())
            } else {
                Err(Error::ConnectFailed)
            }
        }
    }

    pub async fn reconnect_if_required(&self) -> Result<()> {
        match self.status() {
            HostStatus::NotRunning => {
                panic!("A.I.Voice is not running, probably crashed");
            }
            HostStatus::Idle => {
                info!("A.I.Voice is already running and idle");
                return Ok(());
            }
            HostStatus::Busy => {
                info!("A.I.Voice is already running and busy");
                return Ok(());
            }
            _ => {}
        }

        loop {
            info!("Host status: {:?}", self.status());

            if self.status() == HostStatus::NotConnected {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        info!("Connecting to A.I.Voice...");
        self.connect()?;
        info!("Connected to A.I.Voice");

        Ok(())
    }

    pub fn version(&self) -> Result<String> {
        unsafe {
            let ptr = bridge_get_version();
            if ptr.is_null() {
                return Err(Error::VersionFailed);
            }
            let c_str = std::ffi::CStr::from_ptr(ptr);
            let str_slice = c_str.to_str().unwrap();
            let string = str_slice.to_owned();
            bridge_free(ptr as *mut c_char);
            Ok(string)
        }
    }

    pub fn speakers(&self) -> Result<Vec<String>> {
        unsafe {
            let ptr = bridge_get_speakers();
            if ptr.is_null() {
                return Err(Error::SpeakersFailed);
            }
            let speakers = ptr_to_array(ptr);
            bridge_free_array(ptr as *mut *const c_char);
            Ok(speakers)
        }
    }

    pub fn voice_preset_names(&self) -> Result<Vec<String>> {
        unsafe {
            let ptr = bridge_get_voice_preset_names();
            if ptr.is_null() {
                return Err(Error::ApiFailed("VoicePresetNames".to_string()));
            }
            let presets = ptr_to_array(ptr);
            bridge_free_array(ptr as *mut *const c_char);
            Ok(presets)
        }
    }

    pub fn add_voice_preset(&self, preset: VoicePreset) -> Result<()> {
        unsafe {
            let json = serde_json::to_string(&preset).unwrap();
            let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(json.as_str());
            let c_str = std::ffi::CString::new(encoded).unwrap();
            let success = bridge_add_voice_preset(c_str.as_ptr());

            if success {
                Ok(())
            } else {
                Err(Error::ApiFailed("AddVoicePreset".to_string()))
            }
        }
    }

    pub fn get_voice_preset(&self, preset_name: &str) -> Result<VoicePreset> {
        unsafe {
            let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(preset_name);
            let c_str = std::ffi::CString::new(encoded).unwrap();
            let ptr = bridge_get_voice_preset(c_str.as_ptr());
            if ptr.is_null() {
                return Err(Error::ApiFailed("GetVoicePreset".to_string()));
            }
            let c_str = std::ffi::CStr::from_ptr(ptr);
            let (str_slice, _, _) = encoding_rs::SHIFT_JIS.decode(c_str.to_bytes());
            let string = str_slice.to_string();
            bridge_free(ptr as *mut c_char);

            Ok(serde_json::from_str(&string).unwrap())
        }
    }

    pub fn set_text_edit_mode(&self, mode: TextEditMode) -> Result<()> {
        unsafe {
            let success = bridge_set_text_edit_mode(mode as i32);

            if success {
                Ok(())
            } else {
                Err(Error::ApiFailed("TextEditMode=".to_string()))
            }
        }
    }

    pub fn terminate_host(&self) -> Result<()> {
        unsafe {
            // let success = bridge_terminate_host();

            // if success {
            //     Ok(())
            // } else {
            //     Err(Error::TerminateHostFailed)
            // }
            let mut tasks = tasklist().into_iter();
            let Some((_, aivoice_process_id)) = tasks.find(|(task_name, _)| task_name.to_lowercase() == "aivoiceeditor.exe") else {
                return Err(Error::ProcessNotFound);
            };

            info!("A.I.Voice process id: {}", aivoice_process_id);
            if !kill(aivoice_process_id) {
                return Err(Error::TerminateHostFailed);
            }
            info!("Terminated A.I.Voice");
            Ok(())
        }
    }

    pub fn reload_phrase_dictionary(&self) -> Result<()> {
        unsafe {
            let success = bridge_reload_phrase_dictionary();

            if success {
                Ok(())
            } else {
                Err(Error::ApiFailed("ReloadPhraseDictionary".to_string()))
            }
        }
    }

    pub fn set_text(&self, text: &str) -> Result<()> {
        unsafe {
            let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(text);
            let c_str = std::ffi::CString::new(encoded).unwrap();
            let success = bridge_set_text(c_str.as_ptr());

            if success {
                Ok(())
            } else {
                Err(Error::ApiFailed("Text=".to_string()))
            }
        }
    }

    pub fn save_audio_to_file(&self, path: &str) -> Result<()> {
        unsafe {
            let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(path);
            let c_str = std::ffi::CString::new(encoded).unwrap();
            let success = bridge_save_audio_to_file(c_str.as_ptr());

            if success {
                Ok(())
            } else {
                Err(Error::ApiFailed("SaveAudioToFile".to_string()))
            }
        }
    }

    pub fn set_current_voice_preset_name(&self, preset_name: &str) -> Result<()> {
        unsafe {
            let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(preset_name);
            let c_str = std::ffi::CString::new(encoded).unwrap();
            let success = bridge_set_current_voice_preset_name(c_str.as_ptr());

            if success {
                Ok(())
            } else {
                Err(Error::ApiFailed("CurrentVoicePresetName=".to_string()))
            }
        }
    }

    pub fn set_voice_preset(&self, preset: &VoicePreset) -> Result<()> {
        unsafe {
            let json = serde_json::to_string(preset).unwrap();
            let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(json.as_str());
            let c_str = std::ffi::CString::new(encoded).unwrap();
            let success = bridge_set_voice_preset(c_str.as_ptr());

            if success {
                Ok(())
            } else {
                Err(Error::ApiFailed("SetVoicePreset".to_string()))
            }
        }
    }

    fn initialize() -> Result<()> {
        unsafe {
            let success = bridge_com_initialize();

            if success {
                Ok(())
            } else {
                Err(Error::InitializeFailed)
            }
        }
    }

    fn hostname() -> String {
        unsafe {
            let ptr = bridge_initialize_with_hostname();
            if ptr.is_null() {
                panic!("Failed to get hostname");
            }
            let c_str = std::ffi::CStr::from_ptr(ptr);
            let str_slice = c_str.to_str().unwrap();
            let string = str_slice.to_owned();
            bridge_free(ptr as *mut c_char);
            string
        }
    }
}
