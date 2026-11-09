//! Application editing, persistence, launch, and target capabilities.

use super::{AppDraft, AppPickerState, EnvBoxApp, RunSelection};
use crate::message::{LaunchKind, Message, Nav, StatusKind};
use envbox_core::{Application, LaunchTarget};
use envbox_launcher::{format_args, parse_args, RunTarget};
use envbox_storage::validate_application;
use iced::Task;
use std::path::PathBuf;
use uuid::Uuid;

impl EnvBoxApp {
    pub fn select_app(&mut self, id: Uuid) {
        if let Some(app) = self.applications.iter().find(|a| a.id == id) {
            self.app_draft = AppDraft {
                id: Some(app.id),
                name: app.name.clone(),
                kind: match &app.launch {
                    LaunchTarget::Command { .. } => LaunchKind::Command,
                    LaunchTarget::Executable { .. } => LaunchKind::Executable,
                    LaunchTarget::Packaged { .. } => LaunchKind::Packaged,
                },
                path: match &app.launch {
                    LaunchTarget::Command { command } => command.clone(),
                    LaunchTarget::Executable { path } => path.display().to_string(),
                    LaunchTarget::Packaged { aumid, .. } => aumid.clone(),
                },
                args: format_args(&app.arguments),
                work_dir: app
                    .working_directory
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default(),
                profile_id: app.default_profile_id,
                inherit: app.inherit_children,
                chrome_independent_data: app.chrome_independent_data,
                console_host: app.console_host,
                audit: app.audit,
                icon_src: match &app.launch {
                    LaunchTarget::Command { command } => command.clone(),
                    LaunchTarget::Executable { path } => path.display().to_string(),
                    LaunchTarget::Packaged { aumid, .. } => aumid.clone(),
                },
            };
            self.app_saved_draft = self.app_draft.clone();
            self.app_edit_mode = false;
            self.app_advanced = false;
        }
    }

    pub(super) fn begin_new_app(&mut self) -> Task<Message> {
        if self.profiles.is_empty() {
            self.resume_new_app = true;
            self.begin_new_profile();
            self.set_status(StatusKind::Info, "Сначала создайте профиль окружения, сохраните его и затем добавляйте приложения");
            return Task::none();
        }
        self.nav = Nav::Apps;
        self.app_picker = Some(AppPickerState::default());
        self.scan_apps_for_picker()
    }

    pub fn filtered_apps(&self) -> Vec<&Application> {
        let q = self.search.trim().to_ascii_lowercase();
        self.applications
            .iter()
            .filter(|a| {
                q.is_empty()
                    || a.name.to_ascii_lowercase().contains(&q)
                    || match &a.launch {
                        LaunchTarget::Command { command } => {
                            command.to_ascii_lowercase().contains(&q)
                        }
                        LaunchTarget::Executable { path } => {
                            path.to_string_lossy().to_ascii_lowercase().contains(&q)
                        }
                        LaunchTarget::Packaged { aumid, .. } => {
                            aumid.to_ascii_lowercase().contains(&q)
                        }
                    }
            })
            .collect()
    }

    pub(super) fn save_app(&mut self) -> Task<Message> {
        if self.profiles.is_empty() {
            self.set_status(StatusKind::Error, "Ошибка сохранения: сначала создайте профиль окружения");
            return Task::none();
        }
        if !self
            .profiles
            .iter()
            .any(|p| p.id == self.app_draft.profile_id)
        {
            self.set_status(StatusKind::Error, "Ошибка сохранения: профиль по умолчанию не найден");
            return Task::none();
        }
        // AUMID / shell:AppsFolder / WindowsApps / execution-alias paths must
        // become LaunchTarget::Packaged — never a CreateProcess Executable
        // (already-saved legacy entries also normalize at run).
        let raw = match self.app_draft.kind {
            LaunchKind::Command => LaunchTarget::Command {
                command: self.app_draft.path.clone(),
            },
            LaunchKind::Executable | LaunchKind::Packaged => LaunchTarget::Executable {
                path: PathBuf::from(self.app_draft.path.clone()),
            },
        };
        let launch = envbox_launcher::normalize_launch_target(&raw);
        let arguments = parse_args(&self.app_draft.args);
        let working_directory = if self.app_draft.work_dir.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(self.app_draft.work_dir.trim()))
        };
        let app = Application {
            id: self.app_draft.id.unwrap_or_else(Uuid::new_v4),
            name: self.app_draft.name.clone(),
            launch,
            working_directory,
            arguments,
            default_profile_id: self.app_draft.profile_id,
            inherit_children: self.app_draft.inherit,
            chrome_independent_data: self.app_draft.chrome_independent_data,
            console_host: self.app_draft.console_host,
            audit: self.app_draft.audit,
        };
        if let Err(err) = validate_application(&app) {
            self.set_status(StatusKind::Error, format!("Ошибка сохранения: {err}"));
            return Task::none();
        }
        let mut doc = self.store.load_applications().unwrap_or_default();
        if let Some(slot) = doc.applications.iter_mut().find(|a| a.id == app.id) {
            *slot = app.clone();
        } else {
            doc.applications.push(app.clone());
        }
        match self.store.save_applications(&doc) {
            Ok(()) => {
                self.applications = doc.applications;
                self.app_draft.id = Some(app.id);
                self.app_saved_draft = self.app_draft.clone();
                self.app_edit_mode = false;
                self.app_capabilities
                    .insert(app.id, capability_for_app(&app));
                self.set_status(StatusKind::Success, "Приложение сохранено");
                return self.refresh_app_icons();
            }
            Err(err) => self.set_status(StatusKind::Error, format!("Ошибка сохранения: {err}")),
        }
        Task::none()
    }

    pub(super) fn delete_app(&mut self) -> Task<Message> {
        let Some(id) = self.app_draft.id else {
            self.set_status(StatusKind::Error, "Сначала выберите приложение");
            return Task::none();
        };
        let mut doc = self.store.load_applications().unwrap_or_default();
        doc.applications.retain(|a| a.id != id);
        match self.store.save_applications(&doc) {
            Ok(()) => {
                self.applications = doc.applications;
                self.app_capabilities.remove(&id);
                if let Some(next_id) = self.applications.first().map(|app| app.id) {
                    self.select_app(next_id);
                } else {
                    self.app_draft = AppDraft::blank(
                        self.profiles
                            .first()
                            .map(|profile| profile.id)
                            .unwrap_or_default(),
                    );
                    self.app_saved_draft = self.app_draft.clone();
                    self.app_edit_mode = false;
                }
                self.set_status(StatusKind::Success, "Приложение удалено");
            }
            Err(err) => self.set_status(StatusKind::Error, format!("Ошибка удаления: {err}")),
        }
        Task::none()
    }

    pub(super) fn detect_app(
        &mut self,
        app_id: Uuid,
        profile_id: Option<Uuid>,
        site: envbox_core::DetectionSite,
    ) -> Task<Message> {
        self.run_app_intent(
            app_id,
            profile_id
                .map(RunSelection::Profile)
                .unwrap_or(RunSelection::Host),
            Some(site),
        )
    }
    pub(super) fn run_app(&mut self, app_id: Uuid, sel: RunSelection) -> Task<Message> {
        self.run_app_intent(app_id, sel, None)
    }
    fn run_app_intent(
        &mut self,
        app_id: Uuid,
        sel: RunSelection,
        detection: Option<envbox_core::DetectionSite>,
    ) -> Task<Message> {
        if self.update_checking {
            self.set_status(StatusKind::Info, "Идёт обновление; запускайте приложение после его окончания");
            return Task::none();
        }
        let Some(app) = self.applications.iter().find(|a| a.id == app_id).cloned() else {
            self.set_status(StatusKind::Error, "Приложение не найдено");
            return Task::none();
        };
        if !matches!(sel, RunSelection::Host) {
            if let Some(capability) = self.app_capabilities.get(&app_id) {
                if capability.injection == crate::package::InjectionSupport::Unsupported {
                    self.set_status(
                        StatusKind::Error,
                        format!(
                            "«{}» нельзя запустить с профилем окружения. {}",
                            app.name,
                            capability.user_explanation()
                        ),
                    );
                    return Task::none();
                }
            }
        }
        let target = match sel {
            RunSelection::Host => RunTarget::Host,
            RunSelection::Default => {
                let Some(profile) = self
                    .profiles
                    .iter()
                    .find(|p| p.id == app.default_profile_id)
                    .cloned()
                else {
                    self.set_status(StatusKind::Error, "Профиль не найден");
                    return Task::none();
                };
                RunTarget::Profile(profile)
            }
            RunSelection::Profile(id) => {
                let Some(profile) = self.profiles.iter().find(|p| p.id == id).cloned() else {
                    self.set_status(StatusKind::Error, "Профиль не найден");
                    return Task::none();
                };
                RunTarget::Profile(profile)
            }
        };
        let environment = match &target {
            RunTarget::Host => "Локальный хост".into(),
            RunTarget::Profile(profile) => profile.name.clone(),
        };
        let request = Uuid::new_v4();
        self.pending_launches.insert(request);
        self.set_status(
            StatusKind::Info,
            format!("Запуск «{}» с профилем {environment}…", app.name),
        );
        Task::perform(
            async move {
                let (sender, receiver) = iced::futures::channel::oneshot::channel();
                let worker = std::thread::Builder::new()
                    .name("aura-application-launch".into())
                    .spawn(move || {
                        let result =
                            envbox_launcher::InstanceManager::launch(&app, target, detection)
                                .map_err(|error| error.to_string());
                        let _ = sender.send(result);
                    });
                worker.map_err(|error| format!("Не удалось начать запуск приложения: {error}"))?;
                receiver
                    .await
                    .map_err(|_| "Запуск приложения неожиданно прервался".to_string())?
            },
            move |result| {
                Message::AppLaunchFinished(request, environment.clone(), detection, result)
            },
        )
    }

    pub(super) fn finish_app_launch(
        &mut self,
        request: Uuid,
        environment: String,
        detection: Option<envbox_core::DetectionSite>,
        result: Result<envbox_launcher::instance::PendingLaunch, String>,
    ) -> Task<Message> {
        if !self.pending_launches.remove(&request) {
            return Task::none();
        }
        match result {
            Ok(pending) => {
                let Some(id) = self.instances.accept_launch(pending) else {
                    return Task::none();
                };
                self.set_status(
                    StatusKind::Success,
                    if let Some(site) = detection {
                        format!("{site} открыт с профилем {environment} (экземпляр {id})")
                    } else {
                        format!("Экземпляр {id} запущен")
                    },
                );
            }
            Err(err) => self.set_status(StatusKind::Error, format!("Ошибка запуска: {err}")),
        }
        Task::none()
    }

    pub(super) fn create_desktop_shortcut(&mut self, app_id: Uuid) -> Task<Message> {
        let Some(app) = self
            .applications
            .iter()
            .find(|app| app.id == app_id)
            .cloned()
        else {
            self.set_status(StatusKind::Error, "Сначала сохраните приложение");
            return Task::none();
        };
        let root = self.store.root().to_path_buf();
        let icon = self.app_icons.get(&app_id).cloned();
        Task::perform(
            async move {
                let (sender, receiver) = iced::futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    let _ = sender.send(crate::shortcut::create_desktop(
                        &app,
                        &root,
                        icon.as_deref(),
                    ));
                });
                receiver
                    .await
                    .unwrap_or_else(|_| Err("Создание ярлыка неожиданно прервалось".into()))
            },
            Message::AppDesktopShortcutCreated,
        )
    }

    pub(super) fn open_app_location(&mut self, app_id: Uuid) {
        let Some(app) = self.applications.iter().find(|app| app.id == app_id) else {
            self.set_status(StatusKind::Error, "Приложение не найдено");
            return;
        };
        let LaunchTarget::Executable { path } = &app.launch else {
            self.set_status(StatusKind::Info, "У этого способа запуска нет файла программы, расположение которого можно открыть");
            return;
        };
        let Some(folder) = path.parent().filter(|parent| parent.is_dir()) else {
            self.set_status(StatusKind::Error, "Каталог программы не существует");
            return;
        };
        if let Err(err) = std::process::Command::new("explorer.exe")
            .arg(folder)
            .spawn()
        {
            self.set_status(StatusKind::Error, format!("Не удалось открыть расположение программы: {err}"));
        }
    }
}

pub(super) fn capability_for_app(app: &Application) -> crate::package::Capability {
    let target = match &app.launch {
        LaunchTarget::Command { command } => command.as_str(),
        LaunchTarget::Executable { path } => path.to_str().unwrap_or_default(),
        LaunchTarget::Packaged { aumid, .. } => aumid.as_str(),
    };
    crate::package::classify_target(target, &format_args(&app.arguments))
}
