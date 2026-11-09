use iced::widget::{button, column, container, pick_list, row, text, text_input};
use iced::{Alignment, Element, Fill, Length, Padding};

use crate::app::EnvBoxApp;
use crate::font;
use crate::icons::{icon, Icon};
use crate::message::{LaunchKind, Message, NamedId};
use crate::package::{Capability, InjectionSupport};
use crate::theme::*;
use crate::widgets::{
    app_icon_badge, badge, danger_btn, form_field, form_section, kv_row, primary_btn,
    secondary_btn, status_dot, toggle,
};
use envbox_core::ConsoleHost;

use super::{detail_editor_shell, detail_shell, section_title};

pub(super) fn view(app: &EnvBoxApp) -> Element<'_, Message> {
    let selected = app.app_draft.id;
    let editing = app.app_edit_mode || selected.is_none();
    let running_count = selected.map(|id| app.running_count(id)).unwrap_or(0);
    let running = running_count > 0;
    let selected_app = selected.and_then(|id| app.applications.iter().find(|a| a.id == id));

    let app_name = if app.app_draft.name.is_empty() {
        "Приложение не выбрано"
    } else {
        app.app_draft.name.as_str()
    };

    let header_tile: Element<_> = selected
        .and_then(|id| app.app_icons.get(&id))
        .map(|png| {
            container(
                iced::widget::image(iced::widget::image::Handle::from_path(png.clone()))
                    .width(Length::Fixed(36.0))
                    .height(Length::Fixed(36.0)),
            )
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(40.0))
            .style(|_| container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.05, 0.07, 0.10,
                ))),
                border: iced::Border {
                    color: BORDER,
                    width: 1.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            })
            .center_x(40.0)
            .center_y(40.0)
            .into()
        })
        .unwrap_or_else(|| app_icon_badge(&app.app_draft.name, 40.0));

    let header_status = row![
        status_dot(if running { SUCCESS } else { FAINT }),
        text(if running {
            format!("Работает · экземпляров: {running_count}")
        } else {
            "Не запущено".to_string()
        })
        .size(TEXT_CAPTION)
        .color(if running { SUCCESS } else { MUTED })
        .font(font::ui_font()),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let header_action: Element<_> = if let Some(id) = selected {
        let run_button: Element<_> = {
            button(
                row![
                    icon(Icon::Play, INK, 11.0),
                    text("Запустить")
                        .size(TEXT_BODY)
                        .color(INK)
                        .font(font::name_font()),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([8, 14]))
            .style(primary_btn)
            .on_press(Message::AppRunId(id))
            .into()
        };

        if editing {
            run_button
        } else {
            row![
                button(text("Изменить").size(TEXT_BODY).font(font::ui_font()))
                    .padding(Padding::from([8, 10]))
                    .style(secondary_btn)
                    .on_press(Message::AppEdit),
                run_button,
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .into()
        }
    } else {
        container(text("Новое приложение").size(TEXT_BODY).color(ACCENT_TEXT)).into()
    };

    let can_open_location = selected_app
        .map(|a| matches!(&a.launch, envbox_core::LaunchTarget::Executable { .. }))
        .unwrap_or(false);
    let open_loc_btn: Element<_> = if can_open_location {
        button(
            row![
                icon(Icon::ExternalLink, MUTED, 12.0),
                text("Открыть расположение")
                    .size(TEXT_BODY)
                    .color(INK_2)
                    .font(font::ui_font()),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding(Padding::from([8, 10]))
        .style(secondary_btn)
        .on_press(Message::AppOpenLocation)
        .into()
    } else {
        container(text("")).into()
    };

    let title_row = row![
        header_tile,
        column![
            text(app_name)
                .size(TEXT_TITLE)
                .color(INK)
                .font(font::name_font()),
            header_status,
        ]
        .spacing(2)
        .width(Fill),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let action_row = row![
        open_loc_btn,
        iced::widget::Space::with_width(Fill),
        header_action,
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    let mut header = column![title_row, action_row].spacing(8).width(Fill);
    if !editing {
        if let Some(application) = selected_app {
            header = header.push(run_selector(app, application));
        }
    }
    if running {
        header = header.push(
            button(text("Экземпляры").size(TEXT_BODY).font(font::ui_font()))
                .style(secondary_btn)
                .on_press(Message::InstanceFilter(selected)),
        );
    }

    if editing {
        let (content, actions) = app_editor(app);
        detail_editor_shell(column![header, content].spacing(12), actions)
    } else {
        detail_shell(column![header, app_overview(app, selected_app)].spacing(12))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RunChoice {
    label: String,
    profile_id: Option<uuid::Uuid>,
}

impl std::fmt::Display for RunChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

fn run_selector<'a>(
    app: &'a EnvBoxApp,
    application: &'a envbox_core::Application,
) -> Element<'a, Message> {
    let id = application.id;
    let unsupported = app
        .app_capabilities
        .get(&id)
        .is_some_and(|capability| capability.injection == InjectionSupport::Unsupported);
    let mut choices: Vec<_> = app
        .profiles
        .iter()
        .filter(|profile| !unsupported && profile.id != application.default_profile_id)
        .map(|profile| RunChoice {
            label: if app
                .profiles
                .iter()
                .filter(|other| other.name == profile.name)
                .count()
                > 1
            {
                format!("{} ({})", profile.name, &profile.id.to_string()[..8])
            } else {
                profile.name.clone()
            },
            profile_id: Some(profile.id),
        })
        .collect();
    choices.push(RunChoice {
        label: "Запустить без профиля".into(),
        profile_id: None,
    });
    pick_list(choices, None::<RunChoice>, move |choice| {
        Message::AppRunWithId(id, choice.profile_id)
    })
    .placeholder("Другие способы")
    .style(pick_style)
    .menu_style(pick_menu)
    .padding(Padding::from([5, 8]))
    .font(font::ui_font())
    .text_size(TEXT_BODY)
    .width(Fill)
    .into()
}

fn app_overview<'a>(
    app: &'a EnvBoxApp,
    selected_app: Option<&'a envbox_core::Application>,
) -> Element<'a, Message> {
    let Some(id) = app.app_draft.id else {
        return container(
            column![
                text("Приложение не выбрано")
                    .size(TEXT_SECTION)
                    .color(INK)
                    .font(font::name_font()),
                text("Выберите приложение слева или нажмите «Добавить приложение».")
                    .size(TEXT_CAPTION)
                    .color(MUTED)
                    .font(font::ui_font()),
            ]
            .spacing(8),
        )
        .padding(14)
        .width(Fill)
        .style(inner_card_style)
        .into();
    };

    let profile_name = app.profile_name(app.app_draft.profile_id);
    let launch = selected_app
        .map(|a| launch_target_label(&a.launch))
        .unwrap_or_else(|| launch_kind_label(app.app_draft.kind).into());
    let mut settings = column![
        kv_row("Профиль по умолчанию", &profile_name),
        kv_row("Способ запуска", &launch),
    ]
    .spacing(7);
    if let Some(selected_app) = selected_app {
        if let Some((label, target)) = application_target(selected_app) {
            settings = settings.push(kv_row(label, &target));
        }
    }

    let mut content = column![
        section_title("Параметры запуска", "При запуске используются сохранённые настройки"),
        container(settings.padding(10))
            .width(Fill)
            .style(inner_card_style),
    ]
    .spacing(10);
    if selected_app.is_some() {
        content = content.push(
            button(
                row![
                    icon(Icon::ExternalLink, INK_2, 12.0),
                    text("Создать ярлык на рабочем столе")
                        .size(TEXT_BODY)
                        .font(font::ui_font()),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .padding(Padding::from([8, 12]))
            .width(Fill)
            .style(secondary_btn)
            .on_press(Message::AppCreateDesktopShortcut(id)),
        );
    }
    if let Some(capability) = app.app_capabilities.get(&id) {
        if capability.injection != InjectionSupport::Supported {
            content = content.push(compatibility_notice(capability));
        }
    }
    if let Some(application) = selected_app.filter(|value| value.supports_chrome_data()) {
        content = content.push(kv_row(
            "Данные браузера",
            if application.chrome_independent_data {
                "Отдельно для приложения и профиля"
            } else {
                "Из параметров запуска / каталог Chrome по умолчанию"
            },
        ));
        let choices: Vec<_> = app
            .profiles
            .iter()
            .map(|profile| (Some(profile.id), profile.name.clone()))
            .chain(std::iter::once((None, "Локальный хост".into())))
            .flat_map(|(profile_id, label)| {
                envbox_core::DetectionSite::ALL
                    .into_iter()
                    .map(move |site| DetectionChoice {
                        label: format!("{label} · {site}"),
                        profile_id,
                        site,
                    })
            })
            .collect();
        content = content.push(
            text(format!(
                "Профиль по умолчанию: {}",
                app.profile_name(application.default_profile_id)
            ))
            .size(TEXT_CAPTION)
            .color(MUTED),
        );
        content = content.push(
            pick_list(choices, None::<DetectionChoice>, move |choice| {
                Message::AppDetect(id, choice.profile_id, choice.site)
            })
            .placeholder("Выбрать профиль и открыть страницу проверки")
            .style(pick_style)
            .menu_style(pick_menu)
            .padding(Padding::from([7, 8]))
            .font(font::ui_font())
            .text_size(TEXT_BODY)
            .width(Fill),
        );
    }
    if selected_app.is_some_and(crate::package::chromium_renderer_sandbox_limited) {
        content = content.push(
            container(
                text("При запуске нового процесса Chrome / Edge язык веб-страниц задаётся по профилю; уже открытый браузер нужно перезапустить. В песочнице страниц системные сведения всё ещё могут показывать реальные значения.")
                    .size(TEXT_CAPTION)
                    .color(ACCENT_TEXT)
                    .font(font::ui_font()),
            )
            .padding(Padding::from([10, 12]))
            .style(inner_card_style),
        );
    }
    container(content).width(Fill).into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DetectionChoice {
    label: String,
    profile_id: Option<uuid::Uuid>,
    site: envbox_core::DetectionSite,
}
impl std::fmt::Display for DetectionChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LaunchKindChoice(LaunchKind);

impl std::fmt::Display for LaunchKindChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(launch_kind_label(self.0))
    }
}

fn launch_kind_label(kind: LaunchKind) -> &'static str {
    match kind {
        LaunchKind::Command => "Командная строка",
        LaunchKind::Executable => "Исполняемый файл",
        LaunchKind::Packaged => "Пакет приложения Windows",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConsoleHostChoice(ConsoleHost);

impl std::fmt::Display for ConsoleHostChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.0 {
            ConsoleHost::Direct => "Прямой запуск",
            ConsoleHost::Cmd => "Командная строка",
            ConsoleHost::PowerShell => "Windows PowerShell",
            ConsoleHost::PowerShell7 => "PowerShell 7",
            ConsoleHost::WindowsTerminal => "Windows Terminal",
        })
    }
}

fn app_editor(app: &EnvBoxApp) -> (Element<'_, Message>, Element<'_, Message>) {
    let profile_labels: Vec<NamedId> = app
        .profiles
        .iter()
        .map(|p| NamedId {
            name: p.name.clone(),
            id: p.id,
        })
        .collect();
    let selected_profile = profile_labels
        .iter()
        .find(|n| n.id == app.app_draft.profile_id)
        .cloned();
    let (target_label, target_placeholder) = match app.app_draft.kind {
        LaunchKind::Command => ("Команда запуска", "Имя команды или полная командная строка"),
        LaunchKind::Executable => ("Путь к программе", "Полный путь к исполняемому файлу"),
        LaunchKind::Packaged => ("Идентификатор приложения", "AUMID приложения Windows"),
    };
    let mut basic = column![
        form_field(
            "Название приложения",
            text_input("Название приложения", &app.app_draft.name)
                .on_input(Message::AppName)
                .padding(Padding::from([8, 10]))
                .size(TEXT_BODY)
                .style(input_style)
                .font(font::ui_font()),
        ),
        form_field(
            "Способ запуска",
            pick_list(
                LaunchKind::ALL.map(LaunchKindChoice),
                Some(LaunchKindChoice(app.app_draft.kind)),
                |choice| Message::AppLaunchKind(choice.0),
            )
            .style(pick_style)
            .menu_style(pick_menu)
            .padding(Padding::from([8, 10]))
            .text_size(TEXT_BODY)
            .font(font::ui_font())
            .width(Fill),
        ),
        form_field(
            target_label,
            text_input(target_placeholder, &app.app_draft.path)
                .on_input(Message::AppPath)
                .padding(Padding::from([8, 10]))
                .size(TEXT_BODY)
                .style(input_style)
                .font(font::ui_font()),
        ),
    ]
    .spacing(FORM_SPACING);

    // A shell host only applies to commands. Keep an existing incompatible
    // selection visible so the user can correct it instead of losing it.
    if app.app_draft.kind == LaunchKind::Command
        || app.app_draft.console_host != ConsoleHost::Direct
    {
        basic = basic.push(form_field(
            "Терминал",
            pick_list(
                ConsoleHost::ALL.map(ConsoleHostChoice),
                Some(ConsoleHostChoice(app.app_draft.console_host)),
                |choice| Message::AppConsoleHost(choice.0),
            )
            .style(pick_style)
            .menu_style(pick_menu)
            .padding(Padding::from([8, 10]))
            .text_size(TEXT_BODY)
            .font(font::ui_font())
            .width(Fill),
        ));
    }
    if app.app_draft.console_host != ConsoleHost::Direct
        && app.app_draft.kind != LaunchKind::Command
    {
        basic = basic.push(
            text("Выбор терминала доступен только для команд; смените способ запуска или выберите прямой запуск.")
                .size(TEXT_CAPTION)
                .color(DANGER_TEXT)
                .font(font::ui_font()),
        );
    } else if app.app_draft.console_host == ConsoleHost::WindowsTerminal {
        basic = basic.push(
            text("После сохранения команда будет запускаться в Windows Terminal.")
                .size(TEXT_CAPTION)
                .color(MUTED)
                .font(font::ui_font()),
        );
    }

    let mut environment = column![form_field(
        "Профиль по умолчанию",
        pick_list(profile_labels, selected_profile, |n: NamedId| {
            Message::AppProfile(n.id)
        })
        .style(pick_style)
        .menu_style(pick_menu)
        .padding(Padding::from([8, 10]))
        .text_size(TEXT_BODY)
        .font(font::ui_font())
        .width(Fill),
    )]
    .spacing(FORM_SPACING);
    let chrome_supported = app.app_draft.kind == LaunchKind::Executable
        && std::path::Path::new(&app.app_draft.path)
            .file_name()
            .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case("chrome.exe"));
    if chrome_supported || app.app_draft.chrome_independent_data {
        environment = environment
            .push(
                row![
                    text("Отдельный каталог данных браузера")
                        .size(TEXT_BODY)
                        .color(INK_2)
                        .font(font::ui_font())
                        .width(Fill),
                    toggle(
                        app.app_draft.chrome_independent_data,
                        Message::AppChromeIndependentDataChanged,
                    ),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .push(
                text("Хранится отдельно для каждого приложения и профиля; существующие входы и закладки не копируются. profile-directory выбирает вложенный профиль внутри этого каталога.")
                    .size(TEXT_CAPTION)
                    .color(MUTED)
                    .font(font::ui_font()),
            );
    }

    let advanced_toggle = button(
        row![
            text("Дополнительные параметры запуска")
                .size(TEXT_SECTION)
                .color(INK_2)
                .font(font::name_font())
                .width(Fill),
            icon(
                if app.app_advanced {
                    Icon::ChevronDown
                } else {
                    Icon::MoreHorizontal
                },
                MUTED,
                14.0,
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .padding(0)
    .style(win_button)
    .on_press(Message::AppAdvancedToggle);
    let mut advanced = column![advanced_toggle].spacing(FORM_SPACING);
    if app.app_advanced {
        let workdir_input = row![
            text_input("Пусто — каталог программы по умолчанию", &app.app_draft.work_dir)
                .on_input(Message::AppWorkDir)
                .padding(Padding::from([8, 10]))
                .size(TEXT_BODY)
                .style(input_style)
                .font(font::ui_font())
                .width(Fill),
            button(icon(Icon::Folder, MUTED, 14.0))
                .padding(Padding::from([10, 10]))
                .style(secondary_btn)
                .on_press(Message::AppBrowseWorkDir),
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        advanced = advanced
            .push(form_field(
                "Параметры запуска",
                text_input("например, --profile dev", &app.app_draft.args)
                    .on_input(Message::AppArgs)
                    .padding(Padding::from([8, 10]))
                    .size(TEXT_BODY)
                    .style(input_style)
                    .font(font::ui_font()),
            ))
            .push(form_field("Рабочий каталог", workdir_input))
            .push(
                row![
                    text("Дочерние процессы наследуют окружение")
                        .size(TEXT_BODY)
                        .color(INK_2)
                        .font(font::ui_font())
                        .width(Fill),
                    toggle(app.app_draft.inherit, Message::AppInherit),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .push(
                row![
                    text("Вести аудит чтения окружения")
                        .size(TEXT_BODY)
                        .color(INK_2)
                        .font(font::ui_font())
                        .width(Fill),
                    toggle(app.app_draft.audit, Message::AppAudit),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            );
    }
    let save = button(
        row![
            icon(Icon::Check, INK, 14.0),
            text("Сохранить")
                .size(TEXT_BODY)
                .color(INK)
                .font(font::name_font()),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding(Padding::from([8, 16]))
    .width(Fill)
    .style(primary_btn)
    .on_press(Message::AppSave);
    let cancel = button(
        text("Отмена")
            .size(TEXT_BODY)
            .color(INK_2)
            .font(font::ui_font()),
    )
    .padding(Padding::from([8, 12]))
    .style(secondary_btn)
    .on_press(Message::AppEditCancel);
    let mut actions = row![save, cancel]
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Fill);
    if app.app_draft.id.is_some() {
        actions = actions.push(
            button(
                text("Удалить")
                    .size(TEXT_BODY)
                    .color(DANGER_TEXT)
                    .font(font::ui_font()),
            )
            .padding(Padding::from([8, 12]))
            .style(danger_btn)
            .on_press(Message::AppDelete),
        );
    }

    let content = column![
        text("Применится при следующем запуске после сохранения")
            .size(TEXT_CAPTION)
            .color(MUTED)
            .font(font::ui_font()),
        form_section("Основное", basic),
        form_section("Окружение и данные", environment),
        container(advanced)
            .padding(14)
            .width(Fill)
            .style(panel_card_style),
    ]
    .spacing(FORM_SPACING);
    (content.into(), actions.into())
}
fn application_target(app: &envbox_core::Application) -> Option<(&'static str, String)> {
    match &app.launch {
        envbox_core::LaunchTarget::Command { command } => Some(("Команда", command.clone())),
        envbox_core::LaunchTarget::Executable { path } => {
            Some(("Путь к программе", path.display().to_string()))
        }
        envbox_core::LaunchTarget::Packaged { .. } => None,
    }
}

fn launch_target_label(target: &envbox_core::LaunchTarget) -> String {
    match target {
        envbox_core::LaunchTarget::Command { .. } => "Командная строка".into(),
        envbox_core::LaunchTarget::Executable { .. } => "Исполняемый файл".into(),
        envbox_core::LaunchTarget::Packaged { .. } => "Пакет приложения Windows".into(),
    }
}

fn compatibility_notice(cap: &Capability) -> Element<'static, Message> {
    let (bg, fg) = match cap.injection {
        InjectionSupport::Supported => (SUCCESS_BG, SUCCESS_TEXT),
        InjectionSupport::Delayed => (ACCENT_BG, ACCENT_TEXT),
        InjectionSupport::Unsupported => (DANGER_BG, DANGER_TEXT),
    };
    container(
        column![
            badge(cap.badge(), bg, fg),
            text(cap.user_explanation())
                .size(TEXT_CAPTION)
                .color(INK_2)
                .font(font::ui_font()),
        ]
        .spacing(6)
        .padding(10),
    )
    .width(Fill)
    .style(inner_card_style)
    .into()
}
