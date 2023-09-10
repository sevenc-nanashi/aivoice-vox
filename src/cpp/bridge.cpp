#include <Windows.h>
#include <comutil.h>
#include <locale.h>
#include <objbase.h>
#include <shlobj.h>
#include <stdio.h>
#include <stdlib.h>
#include <string>
using namespace std;

#import "libid:5edbd481-4f61-4dc1-b23b-f3b318aa5533" rename_namespace("AIVoiceEditorApi")
using namespace AIVoiceEditorApi;

ITtsControlPtr pTtsControl;

char *wchar_to_char(const wchar_t *wstr) {
  size_t len;
  setlocale(LC_ALL, "Japanese");
  errno_t err = wcstombs_s(&len, NULL, 0, wstr, 0);
  size_t len2 = wcslen(wstr);
  if (err != 0) {
    printf("wcstombs_s failed: %d\n", err);
    return nullptr;
  }
  if (len < len2) {
    printf("Assertion failed: len (%zu) < wcslen(wstr) (%zu)\n", len, len2);
    return nullptr;
  }
  char *ret = (char *)malloc(sizeof(char) * (len + 1));
  wcstombs_s(NULL, ret, len + 1, wstr, len);

  return ret;
}

extern "C" {

bool bridge_com_initialize() {
  HRESULT hr = ::CoInitialize(0);
  if (FAILED(hr)) {
    return false;
  }
  ITtsControlPtr pTtsControl_(__uuidof(TtsControl));
  pTtsControl = pTtsControl_;

  return true;
}

char *bridge_initialize_with_hostname() {
  SAFEARRAY *hosts = pTtsControl->GetAvailableHostNames();
  long lb;
  long ub;
  SafeArrayGetLBound(hosts, 1, &lb);
  SafeArrayGetUBound(hosts, 1, &ub);

  if (lb <= ub) {
    _bstr_t initialhost;
    for (long i = lb; i <= ub; i++) {
      _bstr_t hostname;
      SafeArrayGetElement(hosts, &i, (void **)hostname.GetAddress());
      if (i == 0) {
        initialhost = hostname.copy();
      }
    }
    SafeArrayDestroy(hosts);

    HRESULT hr = pTtsControl->Initialize(initialhost);
    if (FAILED(hr)) {
      return nullptr;
    }

    return wchar_to_char(initialhost);
  } else {
    return nullptr;
  }
}

bool bridge_initialized(char *host) {
  VARIANT_BOOL initialized = pTtsControl->IsInitialized;
  return initialized == VARIANT_TRUE;
}

int32_t bridge_get_status() {
  try {
    HostStatus status = pTtsControl->Status;
    switch (status) {
    case HostStatus_NotRunning:
      return 0;
    case HostStatus_NotConnected:
      return 1;
    case HostStatus_Idle:
      return 2;
    case HostStatus_Busy:
      return 3;
    default:
      return -2;
    }
  } catch (...) {
    return -1;
  }
}

bool bridge_start_host() {
  try {
    HRESULT hr = pTtsControl->StartHost();
    return SUCCEEDED(hr);
  } catch (...) {
    return false;
  }
}

bool bridge_connect() {
  try {
    HRESULT hr = pTtsControl->Connect();
    return SUCCEEDED(hr);
  } catch (...) {
    return false;
  }
}

char *bridge_get_version() {
  try {
    _bstr_t version = pTtsControl->Version;

    return wchar_to_char(version);
  } catch (...) {
    return nullptr;
  }
}

char **bridge_get_speakers() {
  long lb, ub;

  SAFEARRAY *voices = pTtsControl->VoiceNames;

  SafeArrayGetLBound(voices, 1, &lb);
  SafeArrayGetUBound(voices, 1, &ub);

  char **ret = (char **)malloc(sizeof(char *) * (ub - lb + 2));

  for (long i = lb; i <= ub; i++) {
    _bstr_t voice;
    SafeArrayGetElement(voices, &i, (void **)voice.GetAddress());
    char *voice_ptr = wchar_to_char(voice);
    if (voice_ptr == nullptr) {
      printf("voice_ptr is nullptr\n");
      return nullptr;
    }
    ret[i - lb] = voice_ptr;
  }

  ret[ub - lb + 1] = nullptr;
  return ret;
}

void bridge_free_array(char **array) {
  while (*array != nullptr) {
    free(*array);
    array++;
  }
}

bool bridge_set_text_edit_mode(int32_t mode) {
  try {
    pTtsControl->PutTextEditMode((TextEditMode)mode);
    return true;
  } catch (...) {
    return false;
  }
}

char **bridge_get_voice_preset_names() {
  long lb, ub;

  SAFEARRAY *presets = pTtsControl->VoicePresetNames;

  SafeArrayGetLBound(presets, 1, &lb);
  SafeArrayGetUBound(presets, 1, &ub);

  char **ret = (char **)malloc(sizeof(char *) * (ub - lb + 2));

  for (long i = lb; i <= ub; i++) {
    _bstr_t preset;
    SafeArrayGetElement(presets, &i, (void **)preset.GetAddress());
    char *preset_ptr = wchar_to_char(preset);
    if (preset_ptr == nullptr) {
      printf("preset_ptr is nullptr\n");
      return nullptr;
    }
    ret[i - lb] = preset_ptr;
  }

  ret[ub - lb + 1] = nullptr;
  return ret;
}

bool bridge_add_voice_preset(char *json) {
  try {
    _bstr_t json_bstr(json);
    HRESULT hr = pTtsControl->AddVoicePreset(json_bstr);
    return SUCCEEDED(hr);
  } catch (...) {
    return false;
  }
}

char *bridge_get_voice_preset(char *name) {
  try {
    _bstr_t name_bstr(name);
    _bstr_t preset = pTtsControl->GetVoicePreset(name_bstr);
    return wchar_to_char(preset);
  } catch (...) {
    return nullptr;
  }
}

bool bridge_terminate_host() {
  try {
    HRESULT hr = pTtsControl->TerminateHost();
    return SUCCEEDED(hr);
  } catch (...) {
    return false;
  }
}

bool bridge_reload_phrase_dictionary() {
  try {
    HRESULT hr = pTtsControl->ReloadPhraseDictionary();
    return SUCCEEDED(hr);
  } catch (...) {
    return false;
  }
}

bool bridge_set_text(char *text) {
  try {
    _bstr_t text_bstr(text);
    pTtsControl->Text = text_bstr;
    return true;
  } catch (...) {
    return false;
  }
}

bool bridge_save_audio_to_file(char *path) {
  try {
    _bstr_t path_bstr(path);
    HRESULT hr = pTtsControl->SaveAudioToFile(path_bstr);
    return SUCCEEDED(hr);
  } catch (...) {
    return false;
  }
}

bool bridge_set_current_voice_preset_name(char *name) {
  try {
    _bstr_t name_bstr(name);
    pTtsControl->CurrentVoicePresetName = name_bstr;
    return true;
  } catch (...) {
    return false;
  }
}

bool bridge_set_voice_preset(char *json) {
  try {
    _bstr_t json_bstr(json);
    HRESULT hr = pTtsControl->SetVoicePreset(json_bstr);
    return SUCCEEDED(hr);
  } catch (...) {
    return false;
  }
}
void bridge_free(char *ptr) { free(ptr); }
}
