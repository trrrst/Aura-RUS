//! Curated Profile editor option lists (common values only).
//! Display: Russian region names / locale tokens / timezone Russian names.

/// Common regions supported by the Profile editor.
pub const COMMON_REGIONS: &[&str] = &[
    "CN", "US", "TW", "SG", "JP", "PH", "GB", "DE", "KR", "BO", "TR",
];

/// Locale and UI language share the same Windows locale tokens.
pub const COMMON_LOCALES: &[&str] = &[
    "zh-CN", "en-US", "zh-TW", "en-SG", "zh-SG", "ja-JP", "en-PH", "fil-PH", "en-GB", "de-DE",
    "ko-KR", "es-BO", "tr-TR",
];

/// Shared Chinese region labels; unknown codes remain visible unchanged.
pub fn region_label(code: &str) -> String {
    match code.trim().to_ascii_uppercase().as_str() {
        "CN" => "Китай".into(),
        "US" => "США".into(),
        "TW" => "Тайвань".into(),
        "SG" => "Сингапур".into(),
        "JP" => "Япония".into(),
        "PH" => "Филиппины".into(),
        "GB" => "Великобритания".into(),
        "DE" => "Германия".into(),
        "KR" => "Южная Корея".into(),
        "BO" => "Боливия".into(),
        "TR" => "Турция".into(),
        _ => code.to_string(),
    }
}

/// Locale / UI language: show the token itself (no long language labels).
pub fn locale_label(locale: &str) -> String {
    locale.trim().to_string()
}

/// Windows / IANA timezone → short Chinese name. Never leak raw English ids.
/// Covers every pair in `envbox_storage::COMMON_TIMEZONE_PAIRS` + host leftovers.
pub fn timezone_label(tz: &str) -> String {
    let t = tz.trim();
    if t.is_empty() {
        return String::new();
    }
    if t.contains('/') || t.eq_ignore_ascii_case("utc") {
        return iana_label(t);
    }
    windows_tz_label(t)
}

fn iana_label(t: &str) -> String {
    let key = t.trim();
    let mapped = match key {
        "Asia/Shanghai" | "Asia/Chongqing" => Some("Шанхай"),
        "Asia/Urumqi" => Some("Урумчи"),
        "Asia/Hong_Kong" => Some("Гонконг"),
        "Asia/Taipei" => Some("Тайбэй"),
        "Asia/Tokyo" => Some("Токио"),
        "Asia/Seoul" => Some("Сеул"),
        "Asia/Singapore" => Some("Сингапур"),
        "Asia/Manila" => Some("Манила (Филиппины)"),
        "Asia/Jerusalem" => Some("Иерусалим"),
        "Asia/Dubai" => Some("Дубай"),
        "Asia/Kolkata" | "Asia/Calcutta" => Some("Калькутта"),
        "Asia/Dhaka" => Some("Дакка"),
        "Asia/Bangkok" => Some("Бангкок"),
        "America/New_York" => Some("Нью-Йорк"),
        "America/Chicago" => Some("Чикаго"),
        "America/Denver" => Some("Денвер"),
        "America/Los_Angeles" => Some("Лос-Анджелес"),
        "America/Anchorage" => Some("Анкоридж"),
        "America/Halifax" => Some("Галифакс"),
        "America/Bogota" => Some("Богота"),
        "America/La_Paz" => Some("Ла-Пас (Боливия)"),
        "America/Sao_Paulo" => Some("Сан-Паулу"),
        "America/Santiago" => Some("Сантьяго"),
        "America/Mexico_City" => Some("Мехико"),
        "Pacific/Honolulu" => Some("Гонолулу"),
        "Pacific/Auckland" => Some("Окленд"),
        "Europe/London" => Some("Лондон"),
        "Europe/Berlin" => Some("Берлин"),
        "Europe/Paris" => Some("Париж"),
        "Europe/Budapest" => Some("Будапешт"),
        "Europe/Warsaw" => Some("Варшава"),
        "Europe/Kyiv" | "Europe/Kiev" => Some("Киев"),
        "Europe/Istanbul" => Some("Стамбул"),
        "Europe/Moscow" => Some("Москва"),
        "Australia/Sydney" => Some("Сидней"),
        "Australia/Perth" => Some("Перт"),
        "Africa/Johannesburg" => Some("Йоханнесбург"),
        "Africa/Cairo" => Some("Каир"),
        "UTC" | "utc" => Some("UTC"),
        _ => None,
    };
    match mapped {
        Some(s) => s.to_string(),
        // Fallback: last path segment as city-ish token, never "America/New_York".
        None => key.rsplit('/').next().unwrap_or(key).replace('_', " "),
    }
}

fn windows_tz_label(t: &str) -> String {
    let lower = t.trim().to_ascii_lowercase();
    let label = match lower.as_str() {
        "china standard time" => "Китайское время",
        "taipei standard time" => "Тайбэйское время",
        "hong kong standard time" => "Гонконгское время",
        "tokyo standard time" => "Японское время (Токио)",
        "korea standard time" => "Корейское время (Сеул)",
        "singapore standard time" => "Время Сингапура/Филиппин",
        "pacific standard time" => "Тихоокеанское время",
        "us pacific standard time" => "Тихоокеанское время",
        "mountain standard time" => "Горное время",
        "us mountain standard time" => "Горное время",
        "central standard time" => "Центральное время",
        "us central standard time" => "Центральное время",
        "eastern standard time" => "Восточное время",
        "us eastern standard time" => "Восточное время",
        "alaskan standard time" => "Время Аляски",
        "hawaiian standard time" => "Гавайское время",
        "atlantic standard time" => "Атлантическое время",
        "sa pacific standard time" => "Тихоокеанское время Южной Америки",
        "sa western standard time" => "Время Боливии (Ла-Пас)",
        "e. south america standard time" => "Восточное время Южной Америки",
        "e. south america daylight time" => "Восточное время Южной Америки",
        "pacific sa standard time" => "Западное время Южной Америки",
        "mexico standard time" => "Мексиканское время",
        "central america standard time" => "Центральноамериканское время",
        "gmt standard time" => "Британское время",
        "greenwich standard time" => "Время по Гринвичу",
        "w. europe standard time" => "Западноевропейское время (Германия)",
        "romance standard time" => "Центральноевропейское время (запад)",
        "central europe standard time" => "Центральноевропейское время",
        "central european standard time" => "Центральноевропейское стандартное время",
        "fle standard time" => "Восточноевропейское время",
        "gtb standard time" => "Восточноевропейское стандартное время",
        "turkey standard time" => "Турецкое время",
        "russian standard time" => "Московское время",
        "israel standard time" => "Израильское время",
        "arabian standard time" => "Арабское время",
        "arab standard time" => "Арабское время",
        "india standard time" => "Индийское время",
        "bangladesh standard time" => "Бангладешское время",
        "se asia standard time" => "Время Юго-Восточной Азии",
        "myanmar standard time" => "Время Мьянмы",
        "aus eastern standard time" => "Восточноавстралийское время",
        "aus central standard time" => "Центральноавстралийское время",
        "w. australia standard time" => "Западноавстралийское время",
        "new zealand standard time" => "Время Новой Зеландии",
        "south africa standard time" => "Южноафриканское время",
        "egypt standard time" => "Египетское время",
        "morocco standard time" => "Марокканское время",
        "west asia standard time" => "Время Западной Азии",
        "central asia standard time" => "Время Центральной Азии",
        "n. central asia standard time" => "Новосибирское время",
        "north asia standard time" => "Красноярское время",
        "north asia east standard time" => "Иркутское время",
        "yakutsk standard time" => "Якутское время",
        "vladivostok standard time" => "Владивостокское время",
        "sakhalin standard time" => "Сахалинское время",
        "kamchatka standard time" => "Камчатское время",
        "central pacific standard time" => "Центральнотихоокеанское время",
        "west pacific standard time" => "Западнотихоокеанское время",
        "fiji standard time" => "Время Фиджи",
        "tonga standard time" => "Время Тонга",
        "samoa standard time" => "Время Самоа",
        "aleutian standard time" => "Алеутское время",
        "utc" => "UTC",
        _ => "",
    };
    if !label.is_empty() {
        return label.to_string();
    }
    // Host leftovers: strip English timezone boilerplate, then localize region words.
    let mut s = t.trim().to_string();
    for noise in [
        " Standard Time",
        " Daylight Time",
        " Standard",
        " Daylight",
        " Time",
    ] {
        if let Some(pos) = s.to_ascii_lowercase().find(&noise.to_ascii_lowercase()) {
            s.replace_range(pos..pos + noise.len(), "");
        }
    }
    let s = s.trim();
    if s.is_empty() {
        return "UTC".to_string();
    }
    localize_tz_words(s)
}

/// Map leftover English timezone region words to Chinese so the list never looks half-English.
fn localize_tz_words(s: &str) -> String {
    const WORDS: &[(&str, &str)] = &[
        ("E. South America", "Восток Южной Америки"),
        ("S. America", "Южная Америка"),
        ("South America", "Южная Америка"),
        ("North America", "Северная Америка"),
        ("Central America", "Центральная Америка"),
        ("Central Europe", "Центральная Европа"),
        ("Eastern Europe", "Восточная Европа"),
        ("Western Europe", "Западная Европа"),
        ("W. Europe", "Западная Европа"),
        ("E. Europe", "Восточная Европа"),
        ("Northern Europe", "Северная Европа"),
        ("Southern Europe", "Южная Европа"),
        ("Europe", "Европа"),
        ("US Pacific", "США, Тихий океан"),
        ("US Mountain", "США, горное"),
        ("US Central", "США, центр"),
        ("US Eastern", "США, восток"),
        ("Pacific SA", "Запад Южной Америки"),
        ("SA Pacific", "Южная Америка, Тихий океан"),
        ("North Asia East", "Северо-Восточная Азия"),
        ("N. Central Asia", "Северная Центральная Азия"),
        ("North Asia", "Северная Азия"),
        ("Central Asia", "Центральная Азия"),
        ("West Asia", "Западная Азия"),
        ("SE Asia", "Юго-Восточная Азия"),
        ("South Asia", "Южная Азия"),
        ("East Asia", "Восточная Азия"),
        ("Asia", "Азия"),
        ("Pacific", "Тихий океан"),
        ("Atlantic", "Атлантика"),
        ("Indian", "Индийский океан"),
        ("Africa", "Африка"),
        ("Australia", "Австралия"),
        ("AUS Eastern", "Восточная Австралия"),
        ("AUS Central", "Центральная Австралия"),
        ("W. Australia", "Западная Австралия"),
        ("New Zealand", "Новая Зеландия"),
        ("Mountain", "горное"),
        ("Central", "центр"),
        ("Eastern", "восток"),
        ("Western", "запад"),
        ("Alaskan", "Аляска"),
        ("Hawaiian", "Гавайи"),
        ("Aleutian", "Алеутские о-ва"),
        ("Mexico", "Мексика"),
        ("Canada", "Канада"),
        ("Greenland", "Гренландия"),
        ("GMT", "Гринвич"),
        ("Coordinated Universal", "UTC"),
    ];

    let mut out = s.to_string();
    let mut changed = true;
    while changed {
        changed = false;
        for (en, zh) in WORDS {
            let lower = out.to_ascii_lowercase();
            if let Some(pos) = lower.find(&en.to_ascii_lowercase()) {
                out.replace_range(pos..pos + en.len(), zh);
                changed = true;
            }
        }
    }
    let out = out.trim();
    if out.is_empty() {
        "Время".to_string()
    } else if out.to_lowercase().contains("время") {
        out.to_string()
    } else {
        format!("Время: {out}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Region,
    Locale,
    Timezone,
}

/// Generic display label for combo options.
pub fn option_label(kind: OptionKind, value: &str) -> String {
    match kind {
        OptionKind::Region => region_label(value),
        OptionKind::Locale => locale_label(value),
        OptionKind::Timezone => timezone_label(value),
    }
}

pub fn common_regions() -> Vec<String> {
    COMMON_REGIONS.iter().map(|s| (*s).to_string()).collect()
}

pub fn common_locales() -> Vec<String> {
    COMMON_LOCALES.iter().map(|s| (*s).to_string()).collect()
}

/// Filter by value or display label (case-insensitive).
pub fn filter_options<'a>(options: &'a [String], query: &str, kind: OptionKind) -> Vec<&'a str> {
    let q = query.trim().to_lowercase();
    options
        .iter()
        .map(|s| s.as_str())
        .filter(|s| {
            let label = option_label(kind, s);
            // Hide leftovers that would still read as English (user: 不要英文).
            if kind == OptionKind::Timezone && !is_zh_display(&label) {
                return false;
            }
            if q.is_empty() {
                return true;
            }
            s.to_lowercase().contains(&q) || label.to_lowercase().contains(&q)
        })
        .collect()
}

/// True when the display label is Chinese / digit / punctuation (UTC allowed).
fn is_zh_display(label: &str) -> bool {
    let t = label.trim();
    if t.is_empty() {
        return false;
    }
    if t.eq_ignore_ascii_case("utc") {
        return true;
    }
    !t.chars().any(|c| c.is_ascii_alphabetic())
}

/// Ensure the live draft value is always selectable even if not in the list.
pub fn with_current(options: &[String], current: &str) -> Vec<String> {
    let mut out: Vec<String> = options.to_vec();
    let cur = current.trim();
    if !cur.is_empty() && !out.iter().any(|s| s.eq_ignore_ascii_case(cur)) {
        out.insert(0, cur.to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_show_only_cn_us_and_timezone() {
        assert_eq!(region_label("CN"), "中国");
        assert_eq!(region_label("US"), "美国");
        assert_eq!(region_label("JP"), "日本");
        assert_eq!(locale_label("zh-CN"), "zh-CN");
        assert_eq!(locale_label("en-US"), "en-US");
        assert_eq!(timezone_label("China Standard Time"), "中国标准时间");
        assert_eq!(timezone_label("pacific standard time"), "太平洋时间");
        assert_eq!(timezone_label("Atlantic Standard Time"), "大西洋时间");
        assert_eq!(timezone_label("SA Pacific Standard Time"), "南美太平洋时间");
        assert_eq!(
            timezone_label("E. South America Standard Time"),
            "南美东部时间"
        );
        assert_eq!(timezone_label("Pacific SA Standard Time"), "南美西部时间");
        assert_eq!(timezone_label("GMT Standard Time"), "英国时间");
        assert_eq!(timezone_label("Asia/Shanghai"), "上海");
        assert_eq!(timezone_label("America/Los_Angeles"), "洛杉矶");
        // Host leftovers never look like raw English Standard Time ids.
        assert!(!timezone_label("W. Europe Standard Time").contains("Standard"));
        assert!(!timezone_label("Easter Island Standard Time").contains("Standard"));
        assert!(timezone_label("W. Europe Standard Time").contains("西欧"));
        // Unmapped English place names are hidden from the list, not shown raw.
        let host = vec![
            "Pacific Standard Time".to_string(),
            "Easter Island Standard Time".to_string(),
        ];
        assert_eq!(
            filter_options(&host, "", OptionKind::Timezone),
            vec!["Pacific Standard Time"]
        );
    }

    #[test]
    fn filter_matches_cn_us_and_timezone() {
        let regions = vec!["CN".to_string(), "US".to_string()];
        assert_eq!(
            filter_options(&regions, "中国", OptionKind::Region),
            vec!["CN"]
        );
        assert_eq!(
            filter_options(&regions, "美国", OptionKind::Region),
            vec!["US"]
        );

        let opts = vec![
            "Asia/Shanghai".to_string(),
            "America/New_York".to_string(),
        ];
        assert_eq!(
            filter_options(&opts, "上海", OptionKind::Timezone),
            vec!["Asia/Shanghai"]
        );
        assert_eq!(
            filter_options(&opts, "shanghai", OptionKind::Timezone),
            vec!["Asia/Shanghai"]
        );
    }

    #[test]
    fn with_current_prepends_missing_value() {
        let opts = vec!["US".to_string()];
        assert_eq!(with_current(&opts, "JP")[0], "JP");
        assert_eq!(with_current(&opts, "US").len(), 1);
    }
}
