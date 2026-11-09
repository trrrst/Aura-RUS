//! Full-page audit log.

use iced::widget::{button, column, container, pick_list, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length, Padding};

use crate::app::EnvBoxApp;
use crate::font;
use crate::icons::{icon, Icon};
use crate::message::Message;
use crate::theme::{
    self, pick_menu, pick_style, BORDER, FAINT, INK, INK_2, MUTED, SUCCESS_BG, SUCCESS_TEXT,
};
use crate::theme::{TEXT_BODY, TEXT_CAPTION, TEXT_PAGE_TITLE};
use crate::widgets::{badge, empty_state, primary_btn, secondary_btn};

/// `2026-09-25T15:08:59.877Z` → `2026-09-25 15:08:59`.
fn format_ts(ts: &str) -> String {
    let s = ts.trim();
    if s.len() >= 19 {
        let head = &s[..19];
        format!("{} {}", &head[..10], &head[11..])
    } else {
        s.to_string()
    }
}

pub fn view_full(app: &EnvBoxApp) -> Element<'_, Message> {
    use crate::message::Nav;

    let rows_all = app.filtered_audit_events();
    let shown = rows_all.len();
    let total = app.audit_total;
    let mut soft_opts = vec!["Все программы".to_string()];
    soft_opts.extend(app.audit_software_options());
    let filter_sel = if app.audit_filter.trim().is_empty() {
        "Все программы".to_string()
    } else {
        app.audit_filter.clone()
    };

    let header = row![
        column![
            text("Аудит безопасности")
                .size(TEXT_PAGE_TITLE)
                .color(INK)
                .font(font::name_font()),
            text(if app.audit_loading {
                format!("Загрузка записей аудита… · показано {shown} из {total}")
            } else {
                format!("Записи аудита · показано {shown} из {total} · нажмите «Обновить», чтобы получить новые")
            })
            .size(TEXT_CAPTION)
            .color(MUTED)
            .font(font::ui_font()),
        ]
        .spacing(4)
        .width(Fill),
        row![
            pick_list(soft_opts, Some(filter_sel), |s: String| {
                Message::AuditFilter(if s == "Все программы" {
                    String::new()
                } else {
                    s
                })
            })
            .style(pick_style)
            .menu_style(pick_menu)
            .text_size(TEXT_BODY)
            .padding(Padding::from([7, 10]))
            .font(font::ui_font())
            .width(Length::Fixed(180.0)),
            button(
                row![
                    icon(Icon::Folder, INK_2, 12.0),
                    text("Открыть каталог журналов")
                        .size(TEXT_BODY)
                        .color(INK_2)
                        .font(font::ui_font())
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([7, 14]))
            .style(secondary_btn)
            .on_press(Message::OpenAuditDir),
            button(
                row![
                    icon(Icon::Document, INK_2, 12.0),
                    text(if app.audit_loading {
                        "Загрузка…"
                    } else {
                        "Обновить журнал"
                    })
                    .size(TEXT_BODY)
                    .color(INK_2)
                    .font(font::ui_font())
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([7, 14]))
            .style(secondary_btn)
            .on_press_maybe((!app.audit_loading).then_some(Message::AuditRefresh)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .align_y(Alignment::Center);

    let content: Element<_> = if app.audit_loading && shown == 0 {
        empty_state(
            Icon::Document,
            "Загрузка записей аудита",
            "Журнал читается в фоне, можно пользоваться другими страницами.",
            None,
        )
    } else if shown == 0 {
        let guide_btn = button(
            row![
                icon(Icon::Play, INK, 11.0),
                text("Включить аудит в настройках приложения")
                    .size(TEXT_BODY)
                    .color(INK)
                    .font(font::name_font()),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([9, 20]))
        .style(primary_btn)
        .on_press(Message::Nav(Nav::Apps));

        empty_state(
            Icon::Document,
            if total == 0 {
                "Перехваченных обращений к окружению пока нет"
            } else {
                "Для выбранной программы записей нет"
            },
            "Включите в настройках приложения «Аудит чтения окружения (Audit Mode)» — тогда при его работе все обращения к часовому поясу, региону, языку и DNS Windows будут записываться здесь с точностью до миллисекунды.",
            Some(guide_btn.into()),
        )
    } else {
        let table_header = container(
            row![
                text("Время (UTC)")
                    .size(TEXT_CAPTION)
                    .color(FAINT)
                    .font(font::ui_font())
                    .width(Length::Fixed(140.0)),
                text("Программа")
                    .size(TEXT_CAPTION)
                    .color(FAINT)
                    .font(font::ui_font())
                    .width(Length::Fixed(120.0)),
                text("PID")
                    .size(TEXT_CAPTION)
                    .color(FAINT)
                    .font(font::ui_font())
                    .width(Length::Fixed(64.0)),
                text("Перехваченный вызов системного API и подменённый результат")
                    .size(TEXT_CAPTION)
                    .color(FAINT)
                    .font(font::ui_font())
                    .width(Fill),
                text("Статус")
                    .size(TEXT_CAPTION)
                    .color(FAINT)
                    .font(font::ui_font())
                    .width(Length::Fixed(90.0)),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([6, 14]))
        .width(Fill);

        let rows = rows_all.iter().fold(column![].spacing(6), |col, ev| {
            let summary = ev.summary.clone().unwrap_or_default();
            let bg = if ev.virtualized { SUCCESS_BG } else { BORDER };
            let fg = if ev.virtualized { SUCCESS_TEXT } else { MUTED };
            let label = if ev.virtualized {
                "VIRTUALIZED"
            } else {
                "HOST PASS"
            };
            let call_str = if summary.is_empty() {
                format!("{}()", ev.api)
            } else {
                format!("{}() → {}", ev.api, summary)
            };
            let call_str = if ev.n > 1 {
                format!("{call_str} ×{}", ev.n)
            } else {
                call_str
            };
            let soft = app.audit_software_label(ev);

            let card = container(
                row![
                    text(format_ts(&ev.ts_utc))
                        .size(TEXT_CAPTION)
                        .color(MUTED)
                        .font(font::ui_font())
                        .width(Length::Fixed(140.0)),
                    text(soft)
                        .size(TEXT_CAPTION)
                        .color(theme::ACCENT_TEXT)
                        .font(font::ui_font())
                        .width(Length::Fixed(120.0)),
                    text(format!("#{}", ev.pid))
                        .size(TEXT_CAPTION)
                        .color(MUTED)
                        .font(font::ui_font())
                        .width(Length::Fixed(64.0)),
                    text(call_str)
                        .size(TEXT_BODY)
                        .color(INK_2)
                        .font(font::name_font())
                        .width(Fill),
                    badge(label, bg, fg),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([9, 14]))
            .width(Fill)
            .style(theme::inner_card_style);

            col.push(card)
        });

        column![
            table_header,
            scrollable(rows).height(Fill).style(theme::dark_scrollable)
        ]
        .spacing(6)
        .into()
    };

    column![header, content]
        .spacing(16)
        .width(Fill)
        .height(Fill)
        .into()
}
