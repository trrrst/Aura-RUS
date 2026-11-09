//! IP-derived timezone preview; filling changes only the unsaved draft.
use crate::app::proxy_editor::ProxyChoice;
use crate::app::EnvBoxApp;
use crate::font;
use crate::message::Message;
use crate::theme::*;
use crate::widgets::{form_field, form_section, kv_row, primary_btn, secondary_btn};
use iced::widget::{button, column, pick_list, row, text, text_input};
use iced::{Alignment, Element, Fill, Padding};

pub(super) fn view(app: &EnvBoxApp) -> Element<'_, Message> {
    let state = &app.timezone_lookup;
    let query = button(
        text(if state.pending {
            "Запрос…"
        } else {
            "Определить пояс"
        })
        .size(TEXT_BODY)
        .font(font::ui_font()),
    )
    .padding(Padding::from([8, 10]))
    .style(secondary_btn)
    .on_press_maybe((!state.pending).then_some(Message::TimezoneQuery));
    let proxy = &app.profile_draft.proxy;
    let mut body = column![form_field(
        "Прокси Chrome",
        pick_list(
            ProxyChoice::ALL,
            Some(proxy.choice),
            Message::ProfileProxyChoice
        )
        .font(font::ui_font())
        .text_size(TEXT_BODY)
        .style(pick_style)
        .menu_style(pick_menu)
        .padding(Padding::from([8, 10]))
        .width(Fill),
    ),]
    .spacing(14);
    if proxy.choice != ProxyChoice::None {
        body = body.push(
            row![
                form_field(
                    "Сервер",
                    text_input("IPv4 / IPv6 / домен", &proxy.host)
                        .on_input(Message::ProfileProxyHost)
                        .font(font::ui_font())
                        .size(TEXT_BODY)
                        .padding(Padding::from([8, 10]))
                        .style(input_style)
                        .width(Fill),
                ),
                iced::widget::container(form_field(
                    "Порт",
                    text_input("Порт", &proxy.port)
                        .on_input(Message::ProfileProxyPort)
                        .font(font::ui_font())
                        .size(TEXT_BODY)
                        .padding(Padding::from([8, 10]))
                        .style(input_style),
                ))
                .width(85),
            ]
            .spacing(8),
        );
    }
    body = body.push(
        text("Для Chrome и определения внешнего IP, без авторизации; другие приложения и прокси Windows не затрагиваются.")
            .size(TEXT_CAPTION)
            .color(MUTED)
            .font(font::ui_font()),
    );
    body = body.push(
        row![
            text("Часовой пояс по внешнему IP")
                .size(TEXT_BODY)
                .font(font::name_font())
                .width(Fill),
            query,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    );
    body = body.push(
        text(proxy.source_label())
            .size(TEXT_CAPTION)
            .color(MUTED)
            .font(font::ui_font()),
    );
    body = body.push(
        text("Без прокси проверяется текущая сеть Aura; расширения браузера, обход прокси и раздельная маршрутизация сайтов могут давать другой внешний IP.")
            .size(TEXT_CAPTION)
            .color(MUTED)
            .font(font::ui_font()),
    );
    if let Some(error) = &state.error {
        body = body.push(
            text(error)
                .size(TEXT_CAPTION)
                .color(DANGER_TEXT)
                .font(font::ui_font()),
        );
    }
    if state.pending {
        body = body.push(
            button(text("Отменить запрос").size(TEXT_BODY).font(font::ui_font()))
                .padding(Padding::from([8, 10]))
                .style(secondary_btn)
                .on_press(Message::TimezoneDismiss),
        );
    }
    if let Some(result) = &state.result {
        let fill = button(text("Подставить пояс").size(TEXT_BODY).font(font::ui_font()))
            .padding(Padding::from([8, 10]))
            .style(primary_btn)
            .on_press_maybe(result.can_fill.then_some(Message::TimezoneFill));
        body = body.push(iced::widget::horizontal_rule(1));
        body = body.push(
            column![
                kv_row("Источник", &result.source),
                kv_row("Время запроса", &result.queried_at),
                kv_row("Внешний IP", &result.ip.to_string()),
                kv_row(
                    "Регион / город",
                    if result.location.is_empty() {
                        "Не указано сервисом"
                    } else {
                        &result.location
                    }
                ),
                kv_row(
                    "Часовой пояс IANA",
                    if result.iana.is_empty() {
                        "Не указано сервисом"
                    } else {
                        &result.iana
                    }
                ),
                kv_row(
                    "Часовой пояс Windows",
                    result.windows.as_deref().unwrap_or("Не удаётся подставить автоматически")
                ),
                text(&result.status)
                    .size(TEXT_CAPTION)
                    .color(MUTED)
                    .font(font::ui_font()),
                row![
                    fill,
                    button(text("Закрыть").size(TEXT_BODY).font(font::ui_font()))
                        .padding(Padding::from([8, 10]))
                        .style(secondary_btn)
                        .on_press(Message::TimezoneDismiss)
                ]
                .spacing(8),
                text("После подстановки нажмите «Сохранить» — пояс применится к новым экземплярам.")
                    .size(TEXT_CAPTION)
                    .color(MUTED)
                    .font(font::ui_font()),
            ]
            .spacing(12),
        );
    }
    form_section("Прокси и внешний IP", body)
}
