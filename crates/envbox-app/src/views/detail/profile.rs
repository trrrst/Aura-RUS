use iced::widget::{button, column, container, pick_list, row, text, text_input, tooltip};
use iced::{Alignment, Element, Fill, Padding};

use crate::app::EnvBoxApp;
use crate::font;
use crate::icons::{icon, Icon};
use crate::message::{browser_guarantee_label, DnsChoice, Message, WebRtcChoice};
use crate::theme::*;
use crate::widgets::{
    danger_btn, field_label, form_field, form_section, kv_row, primary_btn, searchable_select,
    secondary_btn,
};

use super::{detail_editor_shell, detail_shell, section_title};

mod dns;
mod timezone;

pub(super) fn view(app: &EnvBoxApp) -> Element<'_, Message> {
    let is_new = app.profile_draft.id.is_none();
    let editing = app.profile_edit_mode || is_new;
    let title = if is_new {
        "Новый профиль"
    } else if app.profile_draft.name.is_empty() {
        "Профиль без имени"
    } else {
        app.profile_draft.name.as_str()
    };

    let header_action: Element<'_, Message> = if is_new {
        container(text("Новый").size(TEXT_BODY).color(ACCENT_TEXT)).into()
    } else if editing {
        container(text("")).into()
    } else {
        button(text("Изменить").size(TEXT_BODY).font(font::ui_font()))
            .padding(Padding::from([8, 10]))
            .style(secondary_btn)
            .on_press(Message::ProfileEdit)
            .into()
    };

    let header = row![
        row![
            icon(Icon::Globe, ACCENT_LINE, 15.0),
            column![
                text(title)
                    .size(TEXT_TITLE)
                    .color(INK)
                    .font(font::name_font()),
                text(if is_new {
                    "Заполните настройки окружения, общие для приложений"
                } else {
                    "При запуске приложения окружение процесса готовится по этим настройкам"
                })
                .size(TEXT_CAPTION)
                .color(MUTED)
                .font(font::ui_font()),
            ]
            .spacing(2),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Fill),
        header_action,
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    if editing {
        let (body, actions) = profile_editor(app);
        detail_editor_shell(column![header, body].spacing(20), actions)
    } else {
        detail_shell(column![header, profile_overview(app)].spacing(20))
    }
}

fn profile_overview(app: &EnvBoxApp) -> Element<'_, Message> {
    let p = match app
        .profile_draft
        .id
        .and_then(|id| app.profiles.iter().find(|p| p.id == id))
    {
        Some(p) => p,
        None => {
            return container(
                column![
                    text("Профиль окружения не выбран")
                        .size(TEXT_SECTION)
                        .color(INK)
                        .font(font::name_font()),
                    text("Выберите профиль слева или нажмите «Новый профиль».")
                        .size(TEXT_BODY)
                        .color(MUTED)
                        .font(font::ui_font()),
                ]
                .spacing(8),
            )
            .padding(14)
            .width(Fill)
            .style(inner_card_style)
            .into()
        }
    };

    let webrtc = WebRtcChoice::from_policy(&p.browser.webrtc);

    container(
        column![
            section_title("Обзор профиля", "После сохранения приложения, запущенные с этим профилем, будут видеть эти значения"),
            container(
                column![
                    kv_row("Регион", &crate::options::region_label(&p.locale.region)),
                    kv_row("Locale", &p.locale.locale_name),
                    kv_row("Язык интерфейса", &p.locale.ui_language),
                    kv_row("Часовой пояс Windows", &p.timezone.windows_id),
                    kv_row("Прокси Chrome / проверки", &p.browser.proxy.as_ref().map(|p| p.chrome_server()).unwrap_or_else(|| "Без прокси профиля".into())),
                    text("Прокси используется только для Chrome и определения внешнего IP; сеть других приложений не меняется.")
                        .size(TEXT_CAPTION).color(MUTED).font(font::ui_font()),
                ]
                .spacing(7)
                .padding(10),
            )
            .width(Fill)
            .style(inner_card_style),
            container(
                column![
                    field_label("Дополнительно"),
                    kv_row(
                        "Часовой пояс IANA",
                        if p.timezone.iana_id.is_empty() {
                            "Как у пояса Windows"
                        } else {
                            &p.timezone.iana_id
                        },
                    ),
                    dns::overview(&p.dns),
                    kv_row(
                        "WebRTC",
                        &format!("{} · {}", webrtc, browser_guarantee_label(p.browser.webrtc)),
                    ),
                    kv_row("Переменные окружения", &format!("{} шт.", p.environment.len())),
                ]
                .spacing(7)
                .padding(10),
            )
            .width(Fill)
            .style(inner_card_style),
            container(
                column![
                    field_label("Подмена идентификации"),
                    kv_row(
                        "Имя компьютера",
                        p.identity.computer_name.as_deref().unwrap_or("Как у хоста")
                    ),
                    kv_row(
                        "Имя пользователя",
                        p.identity.user_name.as_deref().unwrap_or("Как у хоста")
                    ),
                    kv_row(
                        "MAC",
                        p.identity.mac_address.as_deref().unwrap_or("Как у хоста")
                    ),
                    kv_row(
                        "MachineGuid",
                        p.identity.machine_guid.as_deref().unwrap_or("Как у хоста")
                    ),
                    text("Влияет только на поддерживаемые способы чтения; реальная учётная запись, сетевые адаптеры и системный реестр не меняются.")
                        .size(TEXT_CAPTION)
                        .color(FAINT)
                        .font(font::ui_font()),
                    text("MAC подменяется для Win32 и локальных CIM-запросов к адаптерам (Get-NetAdapter, Get-CimInstance); удалённые запросы возвращают реальные значения.")
                        .size(TEXT_CAPTION)
                        .color(FAINT)
                        .font(font::ui_font()),
                ]
                .spacing(7)
                .padding(10),
            )
            .width(Fill)
            .style(inner_card_style),

            button(
                text("Удалить профиль")
                    .size(TEXT_CAPTION)
                    .color(DANGER_TEXT)
                    .font(font::ui_font())
            )
            .padding(Padding::from([6, 9]))
            .style(danger_btn)
            .on_press(Message::ProfileDelete),
        ]
        .spacing(10),
    )
    .width(Fill)
    .into()
}

fn profile_editor(app: &EnvBoxApp) -> (Element<'_, Message>, Element<'_, Message>) {
    let region_opts = app.combo_options(crate::message::ComboField::Region);
    let locale_opts = app.combo_options(crate::message::ComboField::Locale);
    let ui_opts = app.combo_options(crate::message::ComboField::Ui);
    let tz_opts = app.combo_options(crate::message::ComboField::Timezone);
    let tz_iana_opts = app.combo_options(crate::message::ComboField::TimezoneIana);

    let basic = form_section(
        "Основное",
        form_field(
            "Название профиля",
            text_input("например, US разработка", &app.profile_draft.name)
                .on_input(Message::ProfileName)
                .size(TEXT_BODY)
                .padding(Padding::from([8, 10]))
                .style(input_style)
                .font(font::ui_font()),
        ),
    );
    let mut language = column![
        form_field(
            "Регион",
            searchable_select(
                crate::message::ComboField::Region,
                &app.profile_draft.region,
                "Выберите регион",
                app.open_combo == Some(crate::message::ComboField::Region),
                &app.combo_query,
                &region_opts,
                Message::ComboQuery,
                Message::ComboPick,
            ),
        ),
        form_field(
            "Региональный формат (Locale)",
            searchable_select(
                crate::message::ComboField::Locale,
                &app.profile_draft.locale,
                "Выберите региональный формат",
                app.open_combo == Some(crate::message::ComboField::Locale),
                &app.combo_query,
                &locale_opts,
                Message::ComboQuery,
                Message::ComboPick,
            ),
        ),
        form_field(
            "Язык интерфейса",
            searchable_select(
                crate::message::ComboField::Ui,
                &app.profile_draft.ui,
                "Выберите язык интерфейса",
                app.open_combo == Some(crate::message::ComboField::Ui),
                &app.combo_query,
                &ui_opts,
                Message::ComboQuery,
                Message::ComboPick,
            ),
        ),
        form_field(
            "Часовой пояс Windows",
            searchable_select(
                crate::message::ComboField::Timezone,
                &app.profile_draft.tz,
                "Выберите часовой пояс Windows",
                app.open_combo == Some(crate::message::ComboField::Timezone),
                &app.combo_query,
                &tz_opts,
                Message::ComboQuery,
                Message::ComboPick,
            ),
        ),
    ]
    .spacing(14);
    if app.profile_advanced {
        language = language.push(form_field(
            "Часовой пояс IANA",
            searchable_select(
                crate::message::ComboField::TimezoneIana,
                &app.profile_draft.tz_iana,
                "Выберите часовой пояс IANA",
                app.open_combo == Some(crate::message::ComboField::TimezoneIana),
                &app.combo_query,
                &tz_iana_opts,
                Message::ComboQuery,
                Message::ComboPick,
            ),
        ));
    }
    let advanced_toggle = button(
        row![
            icon(
                if app.profile_advanced {
                    Icon::ChevronDown
                } else {
                    Icon::MoreHorizontal
                },
                MUTED,
                14.0
            ),
            column![
                text("Дополнительные настройки окружения")
                    .size(TEXT_BODY)
                    .color(INK_2)
                    .font(font::name_font()),
                text("Идентификация, пояс IANA, DNS, WebRTC и переменные")
                    .size(TEXT_CAPTION)
                    .color(MUTED)
                    .font(font::ui_font()),
            ]
            .spacing(4),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .padding(Padding::from([10, 12]))
    .style(secondary_btn)
    .on_press(Message::ProfileAdvancedToggle);
    let mut content = column![
        basic,
        form_section("Язык и часовой пояс", language),
        timezone::view(app),
        advanced_toggle,
    ]
    .spacing(16);
    if app.profile_advanced {
        content = content.push(form_section("Идентификация", column![
                row![
                    text("Пусто — как у хоста").size(TEXT_CAPTION).color(MUTED).font(font::ui_font()),
                    tooltip(icon(Icon::Info, MUTED, 12.0),
                        container(column![
                            text("Подменяет имя компьютера / пользователя Win32, сведения о сетевых адаптерах и MachineGuid в реестре. Реальная учётная запись, адаптеры и идентификатор установки не меняются; CPU, GPU и диски пока не подменяются.").size(TEXT_CAPTION).font(font::ui_font()),
                            text("MAC подменяется при CIM-чтении локальных Win32_NetworkAdapter, Win32_NetworkAdapterConfiguration и MSFT_NetAdapter; WMI/CIM для других полей, GetUserNameEx, Native Registry API и IOCTL устройств пока не подменяются.").size(TEXT_CAPTION).font(font::ui_font()),
                        ].spacing(8)).width(320).padding(12),
                        tooltip::Position::Left,
                    ).style(inner_card_style),
                ].spacing(8).align_y(Alignment::Center),
                form_field(
                    "Имя компьютера",
                    text_input("Имя компьютера ASCII, 1–15 символов", &app.profile_draft.identity.computer_name)
                        .on_input(Message::ProfileComputerName)
                        .size(TEXT_BODY)
                        .padding(Padding::from([8, 10]))
                        .style(input_style)
                        .font(font::ui_font()),
                ),
                form_field(
                    "Имя пользователя",
                    text_input("Буквы, цифры, точка, подчёркивание или дефис", &app.profile_draft.identity.user_name)
                        .on_input(Message::ProfileUserName)
                        .size(TEXT_BODY)
                        .padding(Padding::from([8, 10]))
                        .style(input_style)
                        .font(font::ui_font()),
                ),
                form_field(
                    "MAC",
                    row![
                        text_input("02:AA:BB:CC:DD:EE", &app.profile_draft.identity.mac_address)
                            .on_input(Message::ProfileMacAddress)
                        .size(TEXT_BODY)
                            .padding(Padding::from([8, 10]))
                            .style(input_style)
                            .font(font::ui_font())
                            .width(Fill),
                        button(text("Сгенерировать").size(TEXT_BODY).font(font::ui_font()))
                            .padding(Padding::from([8, 10]))
                            .style(secondary_btn)
                            .on_press(Message::ProfileMacAddressGenerate),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                ),
                form_field(
                    "MachineGuid",
                    row![
                        text_input("UUID установки Windows", &app.profile_draft.identity.machine_guid)
                            .on_input(Message::ProfileMachineGuid)
                        .size(TEXT_BODY)
                            .padding(Padding::from([8, 10]))
                            .style(input_style)
                            .font(font::ui_font())
                            .width(Fill),
                        button(text("Сгенерировать").size(TEXT_BODY).font(font::ui_font()))
                            .padding(Padding::from([8, 10]))
                            .style(secondary_btn)
                            .on_press(Message::ProfileMachineGuidGenerate),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                ),

        ].spacing(14)));
        content = content.push(form_section(
            "DNS и защита сети",
            column![
                form_field(
                    "Режим DNS",
                    pick_list(
                        DnsChoice::ALL,
                        Some(app.profile_draft.dns_mode),
                        Message::ProfileDnsMode
                    )
                    .style(pick_style)
                    .menu_style(pick_menu)
                    .padding(Padding::from([8, 10]))
                    .text_size(TEXT_BODY)
                    .font(font::ui_font())
                    .width(Fill),
                ),
                dns::view(
                    &app.profile_draft.dns_editor,
                    app.profile_draft.dns_mode.to_mode()
                ),
                iced::widget::horizontal_rule(1),
                form_field(
                    "WebRTC",
                    pick_list(
                        WebRtcChoice::ALL,
                        Some(app.profile_draft.webrtc),
                        Message::ProfileWebRtc
                    )
                    .style(pick_style)
                    .menu_style(pick_menu)
                    .padding(Padding::from([8, 10]))
                    .text_size(TEXT_BODY)
                    .font(font::ui_font())
                    .width(Fill),
                ),
                text("Режим «Хост» сохраняет обычное поведение браузера; строгий режим дополнительно включает сетевую защиту в рантайме.")
                    .size(TEXT_CAPTION)
                    .color(MUTED)
                    .font(font::ui_font()),
            ]
            .spacing(14),
        ));
        content = content.push(form_section(
            "Переменные окружения",
            form_field(
                "Список переменных",
                text_input("KEY=VALUE;KEY2=VALUE2", &app.profile_draft.env)
                    .on_input(Message::ProfileEnv)
                    .size(TEXT_BODY)
                    .padding(Padding::from([8, 10]))
                    .style(input_style)
                    .font(font::ui_font()),
            ),
        ));
    }
    let save = button(
        row![
            icon(Icon::Check, INK, 14.0),
            text("Сохранить").size(TEXT_BODY).font(font::name_font())
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([8, 14]))
    .width(Fill)
    .style(primary_btn)
    .on_press(Message::ProfileSave);
    let cancel = button(text("Отмена").size(TEXT_BODY).font(font::ui_font()))
        .padding(Padding::from([8, 12]))
        .style(secondary_btn)
        .on_press(Message::ProfileEditCancel);
    let mut actions = row![save, cancel].spacing(8).align_y(Alignment::Center);
    if app.profile_draft.id.is_some() {
        actions = actions.push(
            button(text("Удалить").size(TEXT_BODY).font(font::ui_font()))
                .padding(Padding::from([8, 10]))
                .style(danger_btn)
                .on_press(Message::ProfileDelete),
        );
    }
    (content.into(), actions.into())
}
fn dns_mode_label(mode: &envbox_core::DnsMode) -> &'static str {
    match mode {
        envbox_core::DnsMode::Host => "Хост",
        envbox_core::DnsMode::VirtualView => "Виртуальное представление",
    }
}
