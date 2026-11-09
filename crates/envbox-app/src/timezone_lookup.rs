//! Actual egress timezone lookup through the captured Profile proxy. No country/offset fallback or retries.
use envbox_core::BrowserProxy;
use std::io::Read;
use std::net::IpAddr;
use std::process::Stdio;
#[cfg(not(windows))]
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct LookupResult {
    pub source: String,
    pub queried_at: String,
    pub ip: IpAddr,
    pub location: String,
    pub iana: String,
    pub windows: Option<String>,
    pub can_fill: bool,
    pub status: String,
}

#[derive(Debug, Clone, Default)]
pub struct LookupState {
    pub generation: u64,
    pub pending: bool,
    pub result: Option<LookupResult>,
    pub error: Option<String>,
}
impl LookupState {
    pub fn invalidate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.pending = false;
        self.result = None;
        self.error = None;
    }
}

pub fn query(proxy: Option<&BrowserProxy>) -> Result<LookupResult, String> {
    if let Some(proxy) = proxy {
        proxy.validate().map_err(|e| e.to_string())?;
    }
    let source = proxy
        .map(|p| format!("Profile {}", p.chrome_server()))
        .unwrap_or_else(|| "Текущая сеть Aura (без прокси профиля)".into());
    let body = request(proxy)?;
    let response: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|_| "Ошибка запроса: сервис изменил формат ответа (некорректный JSON)".to_string())?;
    let field = |name: &str| {
        response
            .get(name)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
    };
    let reported = field("ip");
    let ip = reported
        .parse::<IpAddr>()
        .map_err(|_| "Ошибка запроса: в ответе нет корректного внешнего IP".to_string())?;
    let iana = field("timezone").to_owned();
    let mapped = if iana.is_empty() {
        None
    } else {
        envbox_storage::iana_to_windows_for_region(&iana, Some(field("countryCode")))
    };
    let windows = mapped.map(str::to_owned);
    let can_fill = mapped.is_some_and(envbox_storage::windows_id_exists);
    let status = if iana.is_empty() {
        "Сервис не сообщил конкретный часовой пояс, выберите вручную".into()
    } else if mapped.is_none() {
        "У этого пояса IANA нет однозначного соответствия Windows в CLDR, выберите вручную".into()
    } else if !can_fill {
        "Этот часовой пояс Windows недоступен в системе, выберите вручную".into()
    } else {
        "Найден часовой пояс, который можно подставить".into()
    };
    let location = [field("country"), field("region"), field("city")]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    let queried_at = query_time();
    Ok(LookupResult {
        source,
        queried_at,
        ip,
        location,
        iana,
        windows,
        can_fill,
        status,
    })
}

fn query_time() -> String {
    #[cfg(windows)]
    {
        // Windows GetSystemTime returns UTC, never the selected Profile zone.
        let t = unsafe { windows::Win32::System::SystemInformation::GetSystemTime() };
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
            t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond
        )
    }
    #[cfg(not(windows))]
    {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("Unix {seconds} (UTC)")
    }
}

fn request(proxy: Option<&BrowserProxy>) -> Result<Vec<u8>, String> {
    const LIMIT: u64 = 1024 * 1024;
    let mut command = crate::updater::curl_command()?;
    command.arg("--disable");
    if let Some(proxy) = proxy {
        command.args(["--proxy", &proxy.curl_server(), "--noproxy", ""]);
    }
    let mut child = command
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--max-time",
            "10",
            "--max-filesize",
            "1048576",
            "--proto",
            "=https",
            "--url",
            "https://my.ippure.com/v1/info",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "Ошибка запроса: не удалось запустить системный сетевой запрос".to_string())?;
    let mut body = Vec::new();
    let read = child
        .stdout
        .take()
        .ok_or("Ошибка запроса: не удалось прочитать ответ")?
        .take(LIMIT + 1)
        .read_to_end(&mut body);
    if read.is_err() || body.len() as u64 > LIMIT {
        let _ = child.kill();
        let _ = child.wait();
        return Err("Ошибка запроса: ответ не читается или превышает 1 МиБ".into());
    }
    let status = child
        .wait()
        .map_err(|_| "Ошибка запроса: не удалось прочитать состояние сетевого запроса".to_string())?;
    if !status.success() {
        return Err(match status.code() {
            Some(28) => "Время запроса истекло: общий лимит 10 секунд".into(),
            Some(63) => "Ошибка запроса: ответ превышает 1 МиБ".into(),
            Some(22) => "Ошибка запроса: сервис отклонил запрос или не знает этот IP".into(),
            Some(60) | Some(51) => "Ошибка запроса: проверка HTTPS-сертификата не пройдена".into(),
            _ => "Ошибка запроса: нет связи с сервисом, проверьте сеть и повторите".into(),
        });
    }
    Ok(body)
}
