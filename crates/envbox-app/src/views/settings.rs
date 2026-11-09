//! Window preferences, local data, diagnostics and application updates.

use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length, Padding};

use crate::app::EnvBoxApp;
use crate::font;
use crate::icons::{icon, Icon};
use crate::message::Message;
use crate::theme::{self, INK, INK_2, MUTED, TEXT_BODY, TEXT_CAPTION, TEXT_PAGE_TITLE};
use crate::widgets::{form_section, primary_btn, secondary_btn};

fn setting_row<'a>(
    label: &'static str,
    detail: String,
    action: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![
        column![
            text(label)
                .size(TEXT_BODY)
                .color(INK_2)
                .font(font::ui_font()),
            text(detail)
                .size(TEXT_CAPTION)
                .color(MUTED)
                .font(font::ui_font())
                .width(Fill),
        ]
        .spacing(4)
        .width(Fill),
        action.into(),
    ]
    .spacing(16)
    .align_y(Alignment::Center)
    .into()
}

fn action_button(label: &str, message: Message) -> Element<'static, Message> {
    button(
        text(label.to_string())
            .size(TEXT_BODY)
            .font(font::ui_font()),
    )
    .padding(Padding::from([8, 12]))
    .style(secondary_btn)
    .on_press(message)
    .into()
}

fn info_row(label: &str, value: String) -> Element<'static, Message> {
    row![
        text(label.to_string())
            .size(TEXT_BODY)
            .color(MUTED)
            .font(font::ui_font())
            .width(Length::Fixed(100.0)),
        text(value)
            .size(TEXT_BODY)
            .color(INK_2)
            .font(font::ui_font())
            .width(Fill),
    ]
    .spacing(14)
    .into()
}

fn update_hint(app: &EnvBoxApp) -> String {
    match &app.update_status {
        None => "Проверить вручную последнюю стабильную версию на GitHub".into(),
        Some(Err(error)) => format!("Ошибка проверки: {error}"),
        Some(Ok(crate::updater::CheckResult::UpToDate)) => "Установлена последняя версия".into(),
        Some(Ok(crate::updater::CheckResult::Available {
            tag,
            installer: Some(_),
        })) if app.update_installer_path.is_some() => {
            format!("Найдена {tag}, установщик проверен")
        }
        Some(Ok(crate::updater::CheckResult::Available {
            tag,
            installer: Some(_),
        })) => format!("Найдена {tag}, можно скачать и установить"),
        Some(Ok(crate::updater::CheckResult::Available {
            tag,
            installer: None,
        })) => format!("Найдена {tag}, установщик можно скачать со страницы релизов"),
    }
}

pub fn view(app: &EnvBoxApp) -> Element<'_, Message> {
    let header = column![
        text("Настройки")
            .size(TEXT_PAGE_TITLE)
            .color(INK)
            .font(font::name_font()),
        text("Поведение окна, каталоги данных и обновления")
            .size(TEXT_CAPTION)
            .color(MUTED)
            .font(font::ui_font()),
    ]
    .spacing(6);

    let general = form_section(
        "Общие",
        setting_row(
            "Закрытие главного окна",
            format!(
                "Сейчас: {}. Действует для кнопки закрытия и Alt+F4.",
                app.close_behavior.label()
            ),
            action_button("Изменить", Message::WindowClosePreferenceReset),
        ),
    );

    let audit_summary = if app.audit_loading {
        "Чтение записей аудита…".into()
    } else {
        format!(
            "Всего записей: {}; на странице аудита доступны последние {}.",
            app.audit_total,
            app.audit_events.len()
        )
    };
    let data = form_section(
        "Данные и диагностика",
        column![
            setting_row(
                "Каталог конфигурации",
                app.store.root().display().to_string(),
                action_button("Открыть каталог", Message::OpenConfigDir),
            ),
            info_row(
                "Сохранённые настройки",
                format!(
                    "Приложений: {} · профилей: {}",
                    app.applications.len(),
                    app.profiles.len()
                ),
            ),
            setting_row(
                "Журнал аудита",
                audit_summary,
                action_button("Открыть журнал", Message::OpenAuditDir),
            ),
            setting_row(
                "Диагностика окружения",
                "Показать в отдельном окне, какие язык, часовой пояс и DNS видят программы.".into(),
                button(
                    row![
                        icon(Icon::Play, INK, 12.0),
                        text("Запустить зонд").size(TEXT_BODY).font(font::ui_font()),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding(Padding::from([8, 12]))
                .style(primary_btn)
                .on_press(Message::RunProbe),
            ),
        ]
        .spacing(18),
    );

    let update_label = if app.update_checking {
        "Обработка…"
    } else if app.update_installer_path.is_some() {
        "Установить обновление"
    } else if app.update_asset.is_some() {
        "Скачать и установить"
    } else {
        "Проверить обновления"
    };
    let update_action = if app.update_checking {
        None
    } else if app.update_asset.is_some() {
        Some(Message::InstallUpdate)
    } else {
        Some(Message::CheckUpdate)
    };
    let about = form_section(
        "Об Aura",
        column![
            info_row("Версия приложения", crate::version::product_label()),
            info_row(
                "Ядро",
                format!("{} {}", crate::version::ENGINE_NAME, crate::version::PRODUCT_VERSION),
            ),
            setting_row(
                "Обновления",
                update_hint(app),
                button(text(update_label).size(TEXT_BODY).font(font::ui_font()))
                    .padding(Padding::from([8, 12]))
                    .style(secondary_btn)
                    .on_press_maybe(update_action),
            ),
            row![
                action_button("Страница релизов", Message::OpenReleases),
                action_button("Исходный код", Message::OpenRepository),
            ]
            .spacing(8),
            text("Нативный запуск в Windows · Microsoft Detours. Подмена сведений об окружении действует на дерево запущенных процессов; программы по-прежнему имеют доступ к файлам, сети и правам текущей учётной записи.")
                .size(TEXT_CAPTION)
                .color(MUTED)
                .font(font::ui_font())
                .width(Fill),
        ]
        .spacing(16),
    );

    let content = column![header, general, data, about]
        .spacing(18)
        .width(Fill)
        .max_width(820);
    scrollable(
        container(content)
            .width(Fill)
            .center_x(Fill)
            .padding(Padding::from([22, 24])),
    )
    .height(Fill)
    .style(theme::dark_scrollable)
    .into()
}
