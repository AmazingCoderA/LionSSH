use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    Ru,
    En,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Ru => "ru",
            Lang::En => "en",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Lang::Ru => "Русский",
            Lang::En => "English",
        }
    }
    pub fn from_code(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "en" => Lang::En,
            _ => Lang::Ru,
        }
    }
    pub const ALL: [Lang; 2] = [Lang::Ru, Lang::En];
}

pub fn t(lang: Lang, key: &str) -> &'static str {
    match (lang, key) {
        (_, "app_name") => "LionSSH",
        (Lang::Ru, "ready") => "Готово",
        (Lang::En, "ready") => "Ready",
        (Lang::Ru, "dashboard") => "Дашборд",
        (Lang::En, "dashboard") => "Dashboard",
        (Lang::Ru, "terminal") => "Терминал",
        (Lang::En, "terminal") => "Terminal",
        (Lang::Ru, "scripts") => "Скрипты",
        (Lang::En, "scripts") => "Scripts",
        (Lang::Ru, "scheduler") => "Планировщик",
        (Lang::En, "scheduler") => "Scheduler",
        (Lang::Ru, "proxies") => "Прокси",
        (Lang::En, "proxies") => "Proxies",
        (Lang::Ru, "aliases") => "Алиасы",
        (Lang::En, "aliases") => "Aliases",
        (Lang::Ru, "settings") => "Настройки",
        (Lang::En, "settings") => "Settings",
        (Lang::Ru, "add") => "Добавить",
        (Lang::En, "add") => "Add",
        (Lang::Ru, "edit") => "Изменить",
        (Lang::En, "edit") => "Edit",
        (Lang::Ru, "delete") => "Удалить",
        (Lang::En, "delete") => "Delete",
        (Lang::Ru, "save") => "Сохранить",
        (Lang::En, "save") => "Save",
        (Lang::Ru, "cancel") => "Отмена",
        (Lang::En, "cancel") => "Cancel",
        (Lang::Ru, "run") => "Запуск",
        (Lang::En, "run") => "Run",
        (Lang::Ru, "connect") => "Подключить",
        (Lang::En, "connect") => "Connect",
        (Lang::Ru, "profile") => "Профиль",
        (Lang::En, "profile") => "Profile",
        (Lang::Ru, "language") => "Язык",
        (Lang::En, "language") => "Language",
        (Lang::Ru, "active_profile") => "Активный профиль",
        (Lang::En, "active_profile") => "Active profile",
        (Lang::Ru, "no_profile") => "Без профиля",
        (Lang::En, "no_profile") => "No profile",
        (Lang::Ru, "overview") => "Обзор",
        (Lang::En, "overview") => "Overview",
        (Lang::Ru, "recent_activity") => "Последняя активность",
        (Lang::En, "recent_activity") => "Recent activity",
        (Lang::Ru, "quick_command") => "Быстрая команда",
        (Lang::En, "quick_command") => "Quick command",
        (Lang::Ru, "clear") => "Очистить",
        (Lang::En, "clear") => "Clear",
        (Lang::Ru, "copy_command") => "Копировать команду",
        (Lang::En, "copy_command") => "Copy command",
        (Lang::Ru, "export") => "Экспорт",
        (Lang::En, "export") => "Export",
        (Lang::Ru, "run_now") => "Запустить сейчас",
        (Lang::En, "run_now") => "Run now",
        (Lang::Ru, "toggle") => "Переключить",
        (Lang::En, "toggle") => "Toggle",
        (Lang::Ru, "details") => "Детали",
        (Lang::En, "details") => "Details",
        (Lang::Ru, "save_settings") => "Сохранить настройки",
        (Lang::En, "save_settings") => "Save settings",
        (Lang::Ru, "settings_saved") => "Настройки сохранены",
        (Lang::En, "settings_saved") => "Settings saved",
        _ => "—",
    }
}
