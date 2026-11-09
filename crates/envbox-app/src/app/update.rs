//! GUI orchestration for the explicit GitHub update workflow.

use iced::Task;
use std::path::PathBuf;

use crate::message::{Message, StatusKind};
use crate::updater::{self, CheckResult};
use crate::version::PRODUCT_VERSION;

use super::EnvBoxApp;

impl EnvBoxApp {
    pub(super) fn start_update_check(&mut self) -> Task<Message> {
        if self.update_checking {
            return Task::none();
        }
        self.remove_pending_installer();
        self.update_checking = true;
        self.update_status = None;
        self.update_asset = None;
        self.set_status(StatusKind::Info, "Проверка последней версии на GitHub…");
        Task::perform(
            worker_task(|| updater::check_latest(PRODUCT_VERSION)),
            Message::UpdateResult,
        )
    }

    pub(super) fn finish_update_check(
        &mut self,
        result: Result<CheckResult, String>,
    ) -> Task<Message> {
        self.update_checking = false;
        self.update_installer_path = None;
        self.update_status = Some(result.clone());
        self.update_asset = match &result {
            Ok(CheckResult::Available { installer, .. }) => installer.clone(),
            _ => None,
        };
        match &result {
            Ok(CheckResult::UpToDate) => {
                self.set_status(StatusKind::Success, "Установлена последняя версия");
            }
            Ok(CheckResult::Available {
                tag,
                installer: Some(_),
            }) => {
                self.set_status(StatusKind::Info, format!("Доступна новая версия {tag}, можно скачать и установить"));
            }
            Ok(CheckResult::Available {
                tag,
                installer: None,
            }) => {
                self.set_status(
                    StatusKind::Info,
                    format!("Доступна новая версия {tag}, но нет проверяемого официального установщика"),
                );
            }
            Err(error) => self.set_status(StatusKind::Error, format!("Ошибка проверки обновлений: {error}")),
        }
        Task::none()
    }

    pub(super) fn start_update_install(&mut self) -> Task<Message> {
        if self.update_checking {
            return Task::none();
        }
        if self.has_unsaved_edits() {
            self.set_status(
                StatusKind::Error,
                "Сохраните или отмените изменения, затем устанавливайте обновление Aura",
            );
            return Task::none();
        }

        if self.update_installer_path.is_some() {
            return self.start_pending_installer();
        }
        let Some(asset) = self.update_asset.clone() else {
            self.set_status(StatusKind::Error, "Нет проверенного установщика обновления; сначала проверьте обновления");
            return Task::none();
        };

        self.update_checking = true;
        self.set_status(StatusKind::Info, "Загрузка и проверка официального установщика…");
        Task::perform(
            worker_task(move || updater::download_verified(&asset)),
            Message::UpdateDownloadResult,
        )
    }

    pub(super) fn finish_update_download(
        &mut self,
        result: Result<PathBuf, String>,
    ) -> Task<Message> {
        self.update_checking = false;
        match result {
            Ok(path) => {
                self.update_installer_path = Some(path.clone());
                if self.has_unsaved_edits() {
                    self.set_status(
                        StatusKind::Error,
                        "Установщик проверен, но есть несохранённые изменения; сохраните или отмените их и нажмите «Установить»",
                    );
                    Task::none()
                } else {
                    self.start_pending_installer()
                }
            }
            Err(error) => {
                self.set_status(StatusKind::Error, format!("Ошибка загрузки обновления: {error}"));
                Task::none()
            }
        }
    }

    fn start_pending_installer(&mut self) -> Task<Message> {
        if !self.pending_launches.is_empty() {
            self.set_status(StatusKind::Info, "Приложение запускается; устанавливайте обновление после окончания запуска");
            return Task::none();
        }
        if self.has_unsaved_edits() {
            self.set_status(StatusKind::Error, "Сохраните или отмените изменения, затем запускайте установщик");
            return Task::none();
        }
        if !self.flush_pane_ratios() {
            return Task::none();
        }

        let Some(path) = self.update_installer_path.clone() else {
            self.set_status(StatusKind::Error, "Проверенный установщик не найден, скачайте заново");
            return Task::none();
        };
        let Some(asset) = self.update_asset.clone() else {
            self.set_status(StatusKind::Error, "Сведения об обновлении не найдены, проверьте обновления заново");
            return Task::none();
        };
        self.update_checking = true;
        self.set_status(StatusKind::Info, "Повторная проверка и запуск официального установщика…");
        Task::perform(
            worker_task(move || updater::verify_and_launch(&path, &asset)),
            Message::UpdateInstallResult,
        )
    }

    pub(super) fn finish_update_install(
        &mut self,
        result: Result<(), updater::InstallError>,
    ) -> Task<Message> {
        self.update_checking = false;
        match result {
            Ok(()) => {
                // Keep the source file alive while NSIS is starting and
                // reading it.  It is cleared from GUI state; a later cleanup
                // pass can remove stale files after the installer exits.
                self.update_installer_path = None;
                // Runtime child processes own their staged DLLs and continue
                // after the GUI exits, which preserves the existing close
                // behavior contract.  Exit only after ShellExecute accepted
                // the verified installer.
                self.set_status(StatusKind::Success, "Установщик запущен, Aura сейчас закроется");
                self.flush_pane_ratios();
                iced::exit()
            }
            Err(updater::InstallError::Verification(error)) => {
                self.remove_pending_installer();
                self.set_status(
                    StatusKind::Error,
                    format!("Повторная проверка установщика не пройдена, кэш очищен, скачайте заново: {error}"),
                );
                Task::none()
            }
            Err(updater::InstallError::Launch(error)) => {
                self.set_status(
                    StatusKind::Error,
                    format!("Не удалось запустить установщик, можно повторить: {error}"),
                );
                Task::none()
            }
        }
    }

    pub(super) fn open_releases(&mut self) -> Task<Message> {
        if let Err(error) = updater::open_url(updater::LATEST_RELEASE_URL) {
            self.set_status(
                StatusKind::Error,
                format!("Не удалось открыть страницу релизов GitHub: {error}"),
            );
        }
        Task::none()
    }

    pub(super) fn open_repository(&mut self) -> Task<Message> {
        if let Err(error) = updater::open_url(updater::REPOSITORY_URL) {
            self.set_status(StatusKind::Error, format!("Не удалось открыть репозиторий Aura: {error}"));
        }
        Task::none()
    }

    fn remove_pending_installer(&mut self) {
        if let Some(path) = self.update_installer_path.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Run blocking curl/hash/shell preparation away from Iced's update thread.
fn worker_task<T, F>(work: F) -> impl std::future::Future<Output = T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    async move {
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let _ = tx.send(work());
        });
        rx.await.expect("update worker thread dropped its result")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::updater::ChecksumAsset;

    fn installer_asset() -> updater::InstallerAsset {
        updater::InstallerAsset {
            url: "https://github.com/NGLSL/Aura/releases/download/v0.3.3/aura-setup.exe".into(),
            size: 7,
            sha256: "a".repeat(64),
            checksum: Some(ChecksumAsset {
                url: "https://github.com/NGLSL/Aura/releases/download/v0.3.3/aura-setup.exe.sha256"
                    .into(),
                size: 82,
                sha256: "b".repeat(64),
            }),
        }
    }

    #[test]
    fn successful_check_is_kept_for_settings_view() {
        let (mut app, _) = EnvBoxApp::new();
        let asset = installer_asset();
        let _ = app.finish_update_check(Ok(CheckResult::Available {
            tag: "v0.3.3".into(),
            installer: Some(asset.clone()),
        }));
        assert!(!app.update_checking);
        assert!(matches!(
            app.update_status,
            Some(Ok(CheckResult::Available { .. }))
        ));
        assert_eq!(app.update_asset, Some(asset));
    }

    #[test]
    fn failed_check_is_kept_for_settings_view() {
        let (mut app, _) = EnvBoxApp::new();
        let _ = app.finish_update_check(Err("network timeout".into()));
        assert!(!app.update_checking);
        assert!(matches!(app.update_status, Some(Err(error)) if error == "network timeout"));
        assert!(app.update_asset.is_none());
        assert!(app.status.contains("更新检查失败"));
    }

    #[test]
    fn failed_install_verification_clears_pending_and_allows_redownload() {
        let (mut app, _) = EnvBoxApp::new();
        let path = std::env::temp_dir().join(format!("aura-pending-test-{}", uuid::Uuid::new_v4()));
        std::fs::write(&path, b"payload").unwrap();
        app.update_asset = Some(installer_asset());
        app.update_installer_path = Some(path.clone());
        app.update_checking = true;
        let _ = app.finish_update_install(Err(updater::InstallError::Verification(
            "SHA-256 mismatch".into(),
        )));
        assert!(!app.update_checking);
        assert!(app.update_installer_path.is_none());
        assert!(app.update_asset.is_some());
        assert!(app.status.contains("复核失败"));
        assert!(!path.exists());
    }

    #[test]
    fn failed_install_launch_keeps_pending_path_and_does_not_exit() {
        let (mut app, _) = EnvBoxApp::new();
        let path = std::env::temp_dir().join(format!("aura-pending-test-{}", uuid::Uuid::new_v4()));
        std::fs::write(&path, b"payload").unwrap();
        app.update_asset = Some(installer_asset());
        app.update_installer_path = Some(path.clone());
        app.update_checking = true;
        let _ = app.finish_update_install(Err(updater::InstallError::Launch(
            "user cancelled UAC".into(),
        )));
        assert!(!app.update_checking);
        assert_eq!(app.update_installer_path, Some(path.clone()));
        assert!(app.status.contains("启动失败"));
        let _ = std::fs::remove_file(path);
    }
}
