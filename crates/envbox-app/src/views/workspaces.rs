use crate::app::EnvBoxApp;
use crate::font;
use crate::message::Message;
use crate::theme::{self, DANGER_TEXT, INK, INK_2, MUTED, SUCCESS_TEXT};
use crate::theme::{TEXT_BODY, TEXT_CAPTION, TEXT_SECTION};
use iced::widget::{button, column, container, row, text, tooltip};
use iced::{Alignment, Element, Fill, Padding};

// Legacy route renders management only; startup belongs to the Applications page.
pub fn view(app: &EnvBoxApp) -> Element<'_, Message> {
    let management = &app.workspace_management;
    let mut runs = column![text("Фоновые экземпляры")
        .size(TEXT_SECTION)
        .color(INK)
        .font(font::name_font())]
    .spacing(10);
    if management.busy {
        runs = runs.push(
            text("Обновление состояния запуска…")
                .size(TEXT_BODY)
                .color(MUTED)
                .font(font::ui_font()),
        );
    }
    if let Some(error) = &management.error {
        runs = runs.push(
            text(format!("Ошибка запроса управления запуском: {error}"))
                .size(TEXT_BODY)
                .color(DANGER_TEXT)
                .font(font::ui_font())
                .wrapping(iced::widget::text::Wrapping::Glyph),
        );
    }
    if let Some(notice) = &management.notice {
        runs = runs.push(
            text(notice)
                .size(TEXT_BODY)
                .color(INK_2)
                .font(font::ui_font()),
        );
    }
    if let Some(pending) = &management.pending_run {
        // Recovery is global: an application filter or deleted Profile must not hide it.
        runs = runs.push(technical_info(
            "Результат запуска не подтверждён".into(),
            format!(
                "ID экземпляра: {}\nID области запуска: {}",
                pending.instance_id, pending.container_id
            ),
            INK_2,
        ));
        let mut query = action("Узнать результат запуска");
        if !management.busy {
            query = query.on_press(Message::WorkspaceRunStatus);
        }
        runs = runs.push(query);
    }
    let mut groups = Vec::new();
    for view in &management.instances {
        if app
            .instance_filter
            .is_none_or(|id| view.result.application_id == id)
            && !groups.contains(&view.result.profile_id)
        {
            groups.push(view.result.profile_id);
        }
    }
    for profile_id in groups {
        let instances: Vec<_> = management
            .profile_instances(profile_id)
            .filter(|view| {
                app.instance_filter
                    .is_none_or(|id| view.result.application_id == id)
            })
            .collect();
        let controllable = instances
            .iter()
            .filter(|view| {
                Some(view.result.supervisor_generation.as_str()) == management.generation.as_deref()
                    && !matches!(
                        view.result.state.as_str(),
                        "TrackingLost" | "Exited" | "Stopped" | "Failed"
                    )
            })
            .count();
        let mut heading = row![text(format!("Окружение: {}", profile_label(app, profile_id)))
            .size(TEXT_BODY)
            .color(INK_2)
            .font(font::ui_font())
            .width(Fill)]
        .spacing(8)
        .align_y(Alignment::Center);
        if app.instance_filter.is_none() && controllable > 1 {
            let mut stop = action("Остановить этот экземпляр").style(theme::danger_btn);
            if !management.busy {
                stop = stop.on_press(Message::ManagedProfileStop(profile_id));
            }
            heading = heading.push(stop);
        }
        runs = runs.push(heading);
        for view in instances {
            runs = runs.push(instance_card(app, view));
        }
    }
    container(runs)
        .padding(12)
        .width(Fill)
        .style(theme::panel_card_style)
        .into()
}

fn instance_card<'a>(
    app: &'a EnvBoxApp,
    view: &'a envbox_supervisor::RunView,
) -> Element<'a, Message> {
    let result = &view.result;
    let management = &app.workspace_management;
    let expanded = management.diagnostics_instance == Some(result.instance_id);
    let running = result.state == "Running";
    let lost = result.state == "TrackingLost";
    let terminal = matches!(result.state.as_str(), "Exited" | "Stopped" | "Failed");
    let (status, color) = match result.state.as_str() {
        "Running" => ("Работает", SUCCESS_TEXT),
        "Starting" => ("Запускается", INK_2),
        "Stopping" => ("Останавливается", MUTED),
        "Exited" => ("Завершён", MUTED),
        "Stopped" => ("Остановлено", MUTED),
        "Failed" => ("Ошибка запуска", DANGER_TEXT),
        "TrackingLost" => ("Отслеживание прервано", DANGER_TEXT),
        other => (other, MUTED),
    };
    let application_name = app
        .applications
        .iter()
        .find(|application| application.id == result.application_id)
        .map(|application| application.name.clone())
        .unwrap_or_else(|| "Приложение удалено".into());
    let profile_name = profile_label(app, result.profile_id);
    let mut card = column![row![
        text(application_name)
            .size(TEXT_SECTION)
            .color(INK)
            .font(font::name_font())
            .width(Fill),
        text(status)
            .size(TEXT_BODY)
            .color(color)
            .font(font::ui_font()),
    ]
    .spacing(8)
    .align_y(Alignment::Center),]
    .spacing(10);
    card = card.push(
        text(format!("Окружение: {profile_name}"))
            .size(TEXT_CAPTION)
            .color(MUTED)
            .font(font::ui_font()),
    );
    let observed = result
        .member_runtimes
        .iter()
        .filter(|member| member.environment_facts.is_some())
        .count();
    if running {
        let summary = match &result.environment_facts {
            Some(facts) if facts.config_complete && facts.profile_matches_snapshot => {
                "Основной процесс загрузил нужную конфигурацию"
            }
            Some(_) => "Конфигурация основного процесса совпадает не полностью",
            None => "Сведения об окружении основного процесса ещё не получены",
        };
        let summary = if result.entry_guarantee == envbox_supervisor::ENTRY_CHROME_COMPATIBILITY {
            format!("{summary} · совместимый запуск")
        } else {
            summary.into()
        };
        card = card.push(
            text(summary)
                .size(TEXT_BODY)
                .color(INK_2)
                .font(font::ui_font()),
        );
        if observed < result.known_members.len() {
            card = card.push(
                text("По некоторым дочерним процессам ещё нет данных об окружении")
                    .size(TEXT_CAPTION)
                    .color(MUTED)
                    .font(font::ui_font()),
            );
        }
    } else if lost {
        let recovery = if result.entry_guarantee == envbox_supervisor::ENTRY_CHROME_COMPATIBILITY {
            "Браузер, возможно, ещё работает. Закройте его и обновите состояние; условия восстановления — в диагностике."
        } else {
            "Приложение, возможно, ещё работает, но управлять им сейчас нельзя. Обновите состояние или посмотрите причину в диагностике."
        };
        card = card.push(
            text(recovery)
                .size(TEXT_BODY)
                .color(DANGER_TEXT)
                .font(font::ui_font()),
        );
    }
    let details_label = if expanded {
        "Скрыть подробности"
    } else if result.error.is_some() && !lost {
        "Причина ошибки"
    } else {
        "Диагностика"
    };
    let mut controls =
        row![action(details_label).on_press(Message::WorkspaceDiagnostics(result.instance_id))]
            .spacing(8);
    if !terminal {
        let mut stop = action("Остановить").style(theme::danger_btn);
        if !management.busy
            && Some(result.supervisor_generation.as_str()) == management.generation.as_deref()
            && !lost
        {
            stop = stop.on_press(Message::ManagedInstanceStop(result.instance_id));
        }
        controls = controls.push(stop);
    }
    card = card.push(controls);
    if expanded {
        card = card.push(instance_diagnostics(view));
    }
    container(card)
        .padding(14)
        .width(Fill)
        .style(theme::inner_card_style)
        .into()
}

fn instance_diagnostics(view: &envbox_supervisor::RunView) -> Element<'_, Message> {
    let result = &view.result;
    let observed = result
        .member_runtimes
        .iter()
        .filter(|member| member.environment_facts.is_some())
        .count();
    let mut details = column![
        text(format!(
            "Способ запуска: {}\nГарантия точки входа: {}",
            result.mode,
            envbox_supervisor::entry_guarantee_label(&result.entry_guarantee)
        ))
        .size(TEXT_CAPTION)
        .color(INK_2)
        .font(font::ui_font()),
        text(format!(
            "Данные об окружении получены: {} из {} процессов\nПроцессов в Job сейчас: {:?}",
            observed,
            result.known_members.len(),
            view.process_ids
        ))
        .size(TEXT_CAPTION)
        .color(MUTED)
        .font(font::ui_font()),
    ]
    .spacing(10);
    if let Some(facts) = &result.environment_facts {
        let hooks = facts
            .hooks
            .iter()
            .map(|hook| format!("{}: точек входа — {}", hook.group, hook.attached_api_count))
            .collect::<Vec<_>>()
            .join("\n");
        details = details.push(text(format!("Последние данные корневого процесса\nКонфигурация полная: {}\nПрофиль совпадает со снимком: {}\nУстановленные хуки (не значит, что проверены все точки чтения)\n{}",
            if facts.config_complete { "да" } else { "нет" },
            if facts.profile_matches_snapshot { "да" } else { "нет" },
            if hooks.is_empty() { "нет данных" } else { &hooks }))
            .size(TEXT_CAPTION).color(MUTED).font(font::ui_font()));
    }
    if result.entry_guarantee == envbox_supervisor::ENTRY_CHROME_COMPATIBILITY {
        details = details.push(text("Совместимый запуск проверяет Runtime корневого процесса без строгого контроля точки входа. Дочерние процессы песочницы могут читать реальные сведения компьютера. Выход из Aura не останавливает браузер; после перезапуска Supervisor исходный Job только наблюдается, управление не восстанавливается. Закройте браузер и обновите — отслеживание восстановится, когда Job опустеет; если Job исчез, пока служба была недоступна, состояние прерывания сохранится.")
            .size(TEXT_CAPTION).color(MUTED).font(font::ui_font()));
    }
    if let Some(error) = &result.error {
        details = details.push(
            text(error)
                .size(TEXT_CAPTION)
                .color(DANGER_TEXT)
                .font(font::ui_font())
                .wrapping(iced::widget::text::Wrapping::Glyph),
        );
    }
    details = details.push(text(format!("PID корневого процесса: {}\nЭкземпляр: {}\nApplication: {}\nSupervisor generation: {}\nИдентификатор конфигурации: {}\nСнимок: {}",
        result.root_pid, result.instance_id, result.application_id, result.supervisor_generation,
        result.configuration_id, result.snapshot_digest))
        .size(TEXT_CAPTION).color(MUTED).font(font::ui_font())
        .wrapping(iced::widget::text::Wrapping::Glyph));
    container(details)
        .padding(Padding::from([10, 0]))
        .width(Fill)
        .into()
}

fn action<'a>(label: &'static str) -> iced::widget::Button<'a, Message> {
    button(text(label).size(TEXT_BODY).font(font::ui_font()))
        .padding(Padding::from([9, 14]))
        .style(theme::secondary_btn)
}

fn technical_info(label: String, detail: String, color: iced::Color) -> Element<'static, Message> {
    tooltip(
        text(label)
            .size(TEXT_CAPTION)
            .color(color)
            .font(font::ui_font()),
        text(detail)
            .size(TEXT_CAPTION)
            .color(INK_2)
            .font(font::ui_font()),
        tooltip::Position::Top,
    )
    .style(theme::inner_card_style)
    .into()
}

fn profile_label(app: &EnvBoxApp, id: uuid::Uuid) -> String {
    app.profiles
        .iter()
        .find(|profile| profile.id == id)
        .map(|profile| profile.name.clone())
        .unwrap_or_else(|| format!("Удалённое окружение · {}", crate::widgets::short_id(id)))
}
