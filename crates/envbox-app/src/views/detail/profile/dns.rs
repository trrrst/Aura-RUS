use iced::widget::{
    button, checkbox, column, container, pick_list, row, text, text_input, tooltip, Space,
};
use iced::{Alignment, Element, Fill, Padding};

use crate::app::dns_editor::{DnsEditor, TransportChoice};
use crate::font;
use crate::icons::{icon, Icon};
use crate::message::Message;
use crate::theme::*;
use crate::widgets::{field_label, secondary_btn};

pub(super) fn view(editor: &DnsEditor, mode: envbox_core::DnsMode) -> Element<'_, Message> {
    let explanation = column![
        text("Строгий режим: при ошибке разрешения через поддерживаемые DNS API откат на DNS хоста запрещён.")
            .size(TEXT_CAPTION)
            .font(font::ui_font()),
        text("UDP, TCP, DoT и DoH пробуются по порядку списка, серверы не добавляются автоматически. В режиме «Хост» используется DNS хоста.")
            .size(TEXT_CAPTION)
            .font(font::ui_font()),
        text("Встроенный в приложение DNS/DoH/DoT/DoQ может обходить эти API; подмену в renderer браузера нужно проверять отдельно.")
            .size(TEXT_CAPTION)
            .font(font::ui_font()),
    ]
    .spacing(8);
    let help = tooltip(
        row![
            text("Пояснение")
                .size(TEXT_CAPTION)
                .color(MUTED)
                .font(font::ui_font()),
            icon(Icon::Info, MUTED, 12.0)
        ]
        .spacing(4)
        .align_y(Alignment::Center),
        container(explanation)
            .width(320)
            .padding(12)
            .style(inner_card_style),
        tooltip::Position::Left,
    )
    .gap(8)
    .style(inner_card_style);
    let mut content = column![
        row![
            text("Порядок DNS-серверов")
                .size(TEXT_BODY)
                .color(INK)
                .font(font::name_font())
                .width(Fill),
            help,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        checkbox("Запретить откат на DNS хоста (строгий режим)", editor.strict)
            .on_toggle(Message::ProfileDnsStrict)
            .text_size(TEXT_BODY)
            .font(font::ui_font())
            .width(Fill),
        text(if mode == envbox_core::DnsMode::Host {
            "Сейчас используется DNS хоста; серверы ниже применяются в виртуальном режиме."
        } else {
            "Пробуются сверху вниз; при ошибке — следующий."
        })
        .size(TEXT_CAPTION)
        .color(MUTED)
        .font(font::ui_font()),
    ]
    .spacing(10)
    .width(Fill);
    if editor
        .to_profile(mode)
        .ok()
        .is_some_and(|dns| dns.validate_runtime_support().is_err())
    {
        content = content.push(
            text("Настройку можно сохранить, но текущий Runtime не поддерживает это сочетание протокола, порта или нестрогого режима — запуск будет отклонён.")
                .size(TEXT_CAPTION)
                .color(DANGER_TEXT)
                .font(font::ui_font()),
        );
    }
    if editor
        .upstreams
        .iter()
        .any(envbox_core::DnsUpstream::is_plaintext)
    {
        content = content.push(
            text("Есть незашифрованные серверы: при переходе к UDP / TCP запросы отправляются открытым текстом.")
                .size(TEXT_CAPTION)
                .color(MUTED)
                .font(font::ui_font()),
        );
    }
    if editor.upstreams.is_empty() {
        content = content.push(
            text("DNS-серверы не добавлены")
                .size(TEXT_CAPTION)
                .color(MUTED)
                .font(font::ui_font()),
        );
    }
    for (index, upstream) in editor.upstreams.iter().enumerate() {
        let mut entry = column![upstream_summary(index, upstream)]
            .spacing(8)
            .width(Fill);
        entry = entry.push(
            row![
                button(text("Выше").size(TEXT_BODY).font(font::ui_font()))
                    .padding(Padding::from([6, 10]))
                    .style(secondary_btn)
                    .on_press_maybe((index > 0).then_some(Message::ProfileDnsMove(index, true))),
                button(text("Ниже").size(TEXT_BODY).font(font::ui_font()))
                    .padding(Padding::from([6, 10]))
                    .style(secondary_btn)
                    .on_press_maybe(
                        (index + 1 < editor.upstreams.len())
                            .then_some(Message::ProfileDnsMove(index, false))
                    ),
                Space::new(Fill, 1),
                button(text("Убрать").size(TEXT_BODY).font(font::ui_font()))
                    .padding(Padding::from([6, 10]))
                    .style(secondary_btn)
                    .on_press(Message::ProfileDnsRemove(index)),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        );
        content = content.push(
            container(entry)
                .width(Fill)
                .padding(12)
                .style(inner_card_style),
        );
    }
    let mut draft = column![
        text("Добавить DNS-сервер")
            .size(TEXT_BODY)
            .color(INK_2)
            .font(font::name_font()),
        field_label("Протокол"),
        pick_list(
            TransportChoice::ALL,
            Some(editor.draft.transport),
            Message::ProfileDnsTransport,
        )
        .style(pick_style)
        .menu_style(pick_menu)
        .padding(Padding::from([8, 10]))
        .font(font::ui_font())
        .text_size(TEXT_BODY)
        .width(Fill),
    ]
    .spacing(8)
    .width(Fill);
    if editor.draft.transport == TransportChoice::Doh {
        draft = draft.push(field_label("DoH URL"));
        draft = draft.push(
            text_input("https://resolver.example/dns-query", &editor.draft.url)
                .on_input(Message::ProfileDnsUrl)
                .style(input_style)
                .padding(Padding::from([8, 10]))
                .font(font::ui_font())
                .size(TEXT_BODY),
        );
        draft = draft.push(field_label("IP для подключения · необязательно"));
        draft = draft.push(
            text_input("IP-адреса через запятую", &editor.draft.bootstrap)
                .on_input(Message::ProfileDnsBootstrap)
                .style(input_style)
                .padding(Padding::from([8, 10]))
                .font(font::ui_font())
                .size(TEXT_BODY),
        );
        draft = draft.push(
            text("Если пусто, имя сервиса разрешается через доступные напрямую серверы профиля; если таких нет — ошибка, DNS хоста не используется. Указанный вручную IP заменяет автоматическое разрешение.")
                .size(TEXT_CAPTION)
                .color(MUTED)
                .font(font::ui_font()),
        );
        draft = draft.push(field_label("Проверка TLS-сертификата"));
        draft = draft.push(
            pick_list(
                envbox_core::DnsTlsRevocation::ALL,
                Some(editor.draft.tls_revocation),
                Message::ProfileDnsTlsRevocation,
            )
            .style(pick_style)
            .menu_style(pick_menu)
            .padding(Padding::from([8, 10]))
            .font(font::ui_font())
            .text_size(TEXT_BODY)
            .width(Fill),
        );
    } else {
        draft = draft.push(
            row![
                column![
                    field_label("IP-адрес"),
                    text_input("например, 1.1.1.1", &editor.draft.address)
                        .on_input(Message::ProfileDnsAddress)
                        .style(input_style)
                        .padding(Padding::from([8, 10]))
                        .font(font::ui_font())
                        .size(TEXT_BODY),
                ]
                .spacing(8)
                .width(Fill),
                column![
                    field_label("Порт"),
                    text_input("Порт", &editor.draft.port)
                        .on_input(Message::ProfileDnsPort)
                        .style(input_style)
                        .padding(Padding::from([8, 10]))
                        .font(font::ui_font())
                        .size(TEXT_BODY),
                ]
                .spacing(8)
                .width(76),
            ]
            .spacing(8),
        );
        if editor.draft.transport == TransportChoice::Dot {
            draft = draft.push(field_label("Имя в TLS-сертификате"));
            draft = draft.push(
                text_input("например, cloudflare-dns.com", &editor.draft.server_name)
                    .on_input(Message::ProfileDnsServerName)
                    .style(input_style)
                    .padding(Padding::from([8, 10]))
                    .font(font::ui_font())
                    .size(TEXT_BODY),
            );
        }
    }
    if let Some(error) = editor.draft_error() {
        draft = draft.push(
            text(error)
                .size(TEXT_CAPTION)
                .color(DANGER)
                .font(font::ui_font()),
        );
    }
    draft = draft.push(
        button(text("Добавить сервер").size(TEXT_BODY).font(font::ui_font()))
            .padding(Padding::from([6, 12]))
            .style(secondary_btn)
            .on_press(Message::ProfileDnsAdd),
    );
    content
        .push(column![iced::widget::horizontal_rule(1), draft].spacing(14))
        .into()
}

pub(super) fn overview(profile: &envbox_core::DnsProfile) -> Element<'static, Message> {
    let mut content = column![row![
        field_label("DNS"),
        text(super::dns_mode_label(&profile.mode))
            .size(TEXT_BODY)
            .color(MUTED)
            .font(font::ui_font()),
    ]
    .spacing(8)
    .align_y(Alignment::Center),]
    .spacing(8)
    .width(Fill);
    if profile.mode == envbox_core::DnsMode::Host {
        return content
            .push(
                text("Системный DNS")
                    .size(TEXT_BODY)
                    .color(INK_2)
                    .font(font::ui_font()),
            )
            .into();
    }
    content = content.push(
        text(if profile.strict {
            "Строгий режим · без отката на DNS хоста"
        } else {
            "Нестрогий режим"
        })
        .size(TEXT_BODY)
        .color(MUTED)
        .font(font::ui_font()),
    );
    let upstreams = profile.effective_upstreams();
    if upstreams.is_empty() {
        content = content.push(
            text("DNS-серверы не добавлены")
                .size(TEXT_CAPTION)
                .color(MUTED)
                .font(font::ui_font()),
        );
    }
    for (index, upstream) in upstreams.iter().enumerate() {
        content = content.push(
            container(upstream_summary(index, upstream))
                .padding(Padding::from([4, 0]))
                .width(Fill),
        );
    }
    content.into()
}

fn upstream_summary(
    index: usize,
    upstream: &envbox_core::DnsUpstream,
) -> Element<'static, Message> {
    let (transport, endpoint, details) = match upstream {
        envbox_core::DnsUpstream::Udp { address, port } => (
            "UDP · без шифрования",
            std::net::SocketAddr::new(*address, *port).to_string(),
            None,
        ),
        envbox_core::DnsUpstream::Tcp { address, port } => (
            "TCP · без шифрования",
            std::net::SocketAddr::new(*address, *port).to_string(),
            None,
        ),
        envbox_core::DnsUpstream::Dot {
            address,
            port,
            server_name,
        } => (
            "DoT · шифрование",
            std::net::SocketAddr::new(*address, *port).to_string(),
            Some(format!("Имя сертификата: {server_name}")),
        ),
        envbox_core::DnsUpstream::Doh {
            url,
            bootstrap_ips,
            tls_revocation,
        } => (
            "DoH · шифрование",
            url.clone(),
            Some(format!(
                "{}\nTLS: {tls_revocation}",
                if bootstrap_ips.is_empty() {
                    "IP для подключения: авто; имя сервиса разрешается через доступные напрямую серверы профиля, без DNS хоста."
                        .to_string()
                } else {
                    format!(
                        "IP, заданные вручную:\n{}",
                        bootstrap_ips
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                }
            )),
        ),
    };
    let mut heading = row![text(format!("{} · {transport}", index + 1))
        .size(TEXT_BODY)
        .color(INK_2)
        .font(font::name_font())
        .width(Fill),]
    .spacing(8)
    .align_y(Alignment::Center);
    if let Some(details) = details {
        heading = heading.push(
            tooltip(
                text("Подключение и сертификат")
                    .size(TEXT_CAPTION)
                    .color(MUTED)
                    .font(font::ui_font()),
                container(
                    text(details)
                        .size(TEXT_CAPTION)
                        .font(font::ui_font())
                        .wrapping(iced::widget::text::Wrapping::Glyph),
                )
                .width(300)
                .padding(12)
                .style(inner_card_style),
                tooltip::Position::Left,
            )
            .gap(8)
            .style(inner_card_style),
        );
    }
    column![
        heading,
        text(endpoint)
            .size(TEXT_BODY)
            .color(INK_2)
            .font(font::ui_font())
            .wrapping(iced::widget::text::Wrapping::Glyph)
            .width(Fill),
    ]
    .spacing(4)
    .width(Fill)
    .into()
}
