use serde::{Deserialize, Serialize};
use std::{cell::Cell, collections::BTreeMap, sync::OnceLock};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ru")]
    Russian,
    #[serde(rename = "zh")]
    Chinese,
}

impl Language {
    pub const ALL: [Self; 3] = [Self::English, Self::Russian, Self::Chinese];
    pub fn name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Russian => "Русский",
            Self::Chinese => "简体中文",
        }
    }
    pub fn from_locale(locale: &str) -> Self {
        match locale
            .split(['-', '_', '.', '@', ':'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str()
        {
            "ru" => Self::Russian,
            "zh" => Self::Chinese,
            _ => Self::English,
        }
    }
    pub fn system() -> Self {
        #[cfg(target_os = "windows")]
        {
            #[link(name = "kernel32")]
            unsafe extern "system" {
                fn GetUserDefaultUILanguage() -> u16;
            }
            // Windows LANGID: the low ten bits identify the primary language.
            match unsafe { GetUserDefaultUILanguage() } & 0x3ff {
                0x19 => Self::Russian,
                0x04 => Self::Chinese,
                _ => Self::English,
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            for key in ["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"] {
                if let Ok(locale) = std::env::var(key) {
                    if !locale.is_empty() {
                        return Self::from_locale(&locale);
                    }
                }
            }
            Self::English
        }
    }
    pub fn startup(saved: Option<Self>) -> Self {
        saved.unwrap_or_else(Self::system)
    }
}

// egui runs on the UI thread. Thread-local selection also isolates parallel tests.
thread_local! { static CURRENT: Cell<Language> = const { Cell::new(Language::English) }; }
pub fn current() -> Language {
    CURRENT.get()
}
pub fn set_language(language: Language) {
    CURRENT.set(language);
}

pub fn tr(russian: &'static str) -> &'static str {
    static CATALOG: OnceLock<BTreeMap<String, [String; 2]>> = OnceLock::new();
    let language = current();
    if language == Language::Russian {
        return russian;
    }
    let catalog = CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("locales.json")).expect("valid translation catalog")
    });
    match catalog.get(russian) {
        Some(value) => &value[if language == Language::Chinese { 1 } else { 0 }],
        None => russian,
    }
}

// Literal templates keep Rust's formatting checks and named captures intact.
#[macro_export]
macro_rules! tf {
    ($ru:literal, $en:literal, $zh:literal $($args:tt)*) => {
        match $crate::i18n::current() {
            $crate::i18n::Language::Russian => format!($ru $($args)*),
            $crate::i18n::Language::English => format!($en $($args)*),
            $crate::i18n::Language::Chinese => format!($zh $($args)*),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn locales_and_fallback() {
        for locale in ["ru", "ru-RU", "RU_ru.UTF-8"] {
            assert_eq!(Language::from_locale(locale), Language::Russian);
        }
        for locale in ["zh-CN", "zh_TW.UTF-8", "zh-Hans"] {
            assert_eq!(Language::from_locale(locale), Language::Chinese);
        }
        for locale in ["", "de-DE", "C", "en-US", "ja-JP"] {
            assert_eq!(Language::from_locale(locale), Language::English);
        }
        assert_eq!(Language::default(), Language::English);
        assert_eq!(
            Language::startup(Some(Language::Chinese)),
            Language::Chinese
        );
    }
    #[test]
    fn switching_and_formatted_messages() {
        for (language, expected) in [
            (Language::English, "Open"),
            (Language::Russian, "Открыть"),
            (Language::Chinese, "打开"),
        ] {
            set_language(language);
            assert_eq!(tr("Открыть"), expected);
            let count = 3;
            let value = crate::tf!("Иконок: {count}", "Icons: {count}", "图标：{count}");
            assert!(value.contains('3'));
        }
        set_language(Language::English);
    }
    #[test]
    fn old_preferences_and_language_roundtrip() {
        let old: crate::storage::Preferences = serde_json::from_str(r#"{"favorites":[]}"#).unwrap();
        assert_eq!(old.language, None);
        for language in Language::ALL {
            let p = crate::storage::Preferences {
                language: Some(language),
                ..Default::default()
            };
            let copy: crate::storage::Preferences =
                serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
            assert_eq!(copy.language, Some(language));
        }
    }
}
