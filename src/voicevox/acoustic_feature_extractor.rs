use derive_getters::Getters;
use derive_new::new;
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[rustfmt::skip]
const PHONEME_LIST: &[&str] = &[
    "pau",
    "A",
    "E",
    "I",
    "N",
    "O",
    "U",
    "a",
    "b",
    "by",
    "ch",
    "cl",
    "d",
    "dy",
    "e",
    "f",
    "g",
    "gw",
    "gy",
    "h",
    "hy",
    "i",
    "j",
    "k",
    "kw",
    "ky",
    "m",
    "my",
    "n",
    "ny",
    "o",
    "p",
    "py",
    "r",
    "ry",
    "s",
    "sh",
    "t",
    "ts",
    "ty",
    "u",
    "v",
    "w",
    "y",
    "z",
];

static PHONEME_MAP: Lazy<HashMap<&str, i64>> = Lazy::new(|| {
    let mut m = HashMap::new();
    for (i, s) in PHONEME_LIST.iter().enumerate() {
        m.insert(*s, i as i64);
    }
    m
});

#[derive(Debug, Clone, PartialEq, new, Default, Getters)]
pub struct OjtPhoneme {
    phoneme: String,
    #[allow(dead_code)]
    start: f32,
    #[allow(dead_code)]
    end: f32,
}

impl OjtPhoneme {
    pub fn num_phoneme() -> usize {
        PHONEME_MAP.len()
    }

    pub fn space_phoneme() -> String {
        "pau".into()
    }

    pub fn phoneme_id(&self) -> i64 {
        if self.phoneme.is_empty() {
            -1
        } else {
            *PHONEME_MAP.get(&self.phoneme.as_str()).unwrap()
        }
    }

    pub fn convert(phonemes: &[OjtPhoneme]) -> Vec<OjtPhoneme> {
        let mut phonemes = phonemes.to_owned();
        if let Some(first_phoneme) = phonemes.first_mut() {
            if first_phoneme.phoneme.contains("sil") {
                first_phoneme.phoneme = OjtPhoneme::space_phoneme();
            }
        }
        if let Some(last_phoneme) = phonemes.last_mut() {
            if last_phoneme.phoneme.contains("sil") {
                last_phoneme.phoneme = OjtPhoneme::space_phoneme();
            }
        }
        phonemes
    }
}
