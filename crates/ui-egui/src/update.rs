//! Update notifications: the host looks up the latest release ([`Services::check_update`]); when it
//! is newer than this build, a bar under the ribbon says so, with Update (download it, or reload
//! the web app) and Later. Help › Check for Updates asks on demand and also says when there is
//! nothing new.
//!
//! [`Services::check_update`]: crate::Services::check_update

use std::sync::{Arc, Mutex};

use egui::{Stroke, Ui, vec2};

use crate::WordApp;
use crate::theme::{Tokens, medium, regular};

/// The latest release (GitHub's API; it answers browsers from any origin).
pub const LATEST_RELEASE_API: &str = "https://api.github.com/repos/abahvictor360-sketch/Wordpadx/releases/latest";

/// Where this copy of the app runs, which decides what Update does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    /// A browser or an installed web app: reload.
    Web,
    /// The Android app: download the new APK.
    Android,
    /// The iPhone/iPad app: the release page (the build needs signing before it installs).
    Ios,
}

/// Read GitHub's latest-release JSON (`tag_name`, `html_url`, `assets[].name` and
/// `browser_download_url`) into what the update bar needs on `platform`.
pub fn release_from_json(json: &str, platform: Platform) -> Result<UpdateInfo, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("bad release data: {e}"))?;
    let tag = v["tag_name"].as_str().filter(|t| !t.is_empty()).ok_or("the release has no version")?;
    let page = v["html_url"].as_str().filter(|u| u.starts_with("https://")).ok_or("the release has no page")?.to_string();
    let asset = |pred: &dyn Fn(&str) -> bool| {
        v["assets"].as_array().into_iter().flatten().find_map(|a| {
            let (name, url) = (a["name"].as_str()?, a["browser_download_url"].as_str()?);
            (pred(&name.to_ascii_lowercase()) && url.starts_with("https://")).then(|| url.to_string())
        })
    };
    let action = match platform {
        Platform::Web => UpdateAction::Reload,
        Platform::Android => UpdateAction::Open(asset(&|n| n.ends_with(".apk")).unwrap_or_else(|| page.clone())),
        Platform::Ios => UpdateAction::Open(page.clone()),
    };
    Ok(UpdateInfo { version: tag.trim_start_matches(['v', 'V']).to_string(), action, page })
}

/// How often the app looks for an update while it runs (milliseconds): at start, then every 6 h.
const CHECK_EVERY_MS: f64 = 6.0 * 60.0 * 60.0 * 1000.0;
/// The first look waits a little, so it doesn't compete with opening the app.
const FIRST_CHECK_AFTER_MS: f64 = 4_000.0;

/// A newer release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateInfo {
    /// Its version, `0.6.2` (no `v`).
    pub version: String,
    /// What Update does.
    pub action: UpdateAction,
    /// The release page (notes and every download).
    pub page: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateAction {
    /// The web app: reload it, which loads the new version.
    Reload,
    /// Open this download in the system browser (the Android APK, the iOS build).
    Open(String),
}

/// The answer to a check, written by the host.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum UpdateResult {
    #[default]
    None,
    /// The latest release (newer or not; the app compares), or why the check failed.
    Done(Result<UpdateInfo, String>),
}

/// Where the host puts the answer ([`crate::Services::update_slot`]).
pub type UpdateSlot = Arc<Mutex<UpdateResult>>;

/// The update state the app keeps.
#[derive(Debug, Default)]
pub struct UpdateUi {
    /// A newer release to tell the user about.
    pub available: Option<UpdateInfo>,
    /// The version the user said Later to (not shown again this session).
    dismissed: Option<String>,
    /// When the last check was asked for (ms; `None` before the first).
    last_check: Option<f64>,
    /// The user asked (Help › Check for Updates): say the answer even if there is nothing new.
    manual: bool,
    /// When the app started (ms), for the first check.
    started_at: Option<f64>,
}

/// Is version `latest` newer than `current`? Both `X.Y.Z` with an optional `v` and an optional
/// `-pre` suffix; a release is newer than its own pre-releases. Unparsable versions are never
/// newer, so junk never nags.
pub fn is_newer(latest: &str, current: &str) -> bool {
    match (parse(latest), parse(current)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

/// `(major, minor, patch, is_release)`.
fn parse(v: &str) -> Option<(u64, u64, u64, bool)> {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let (core, pre) = match v.split_once(['-', '+']) {
        Some((c, rest)) => (c, !rest.is_empty()),
        None => (v, false),
    };
    let mut it = core.split('.').map(|p| p.parse::<u64>().ok());
    let (a, b, c) = (it.next()??, it.next().unwrap_or(Some(0))?, it.next().unwrap_or(Some(0))?);
    if it.next().is_some() {
        return None;
    }
    Some((a, b, c, !pre))
}

impl UpdateUi {
    /// The first check hasn't been made yet.
    pub(crate) fn first_check_pending(&self) -> bool {
        self.last_check.is_none()
    }
}

impl WordApp {
    /// Ask the host to look for an update now (`manual`: the user asked, so say the answer).
    pub(crate) fn check_for_update(&mut self, manual: bool) -> bool {
        let Some(check) = &self.services.check_update else {
            if manual {
                self.status(tl!("This version of Wordpadx can't check for updates. See the Releases page on GitHub."));
            }
            return false;
        };
        check();
        self.update.last_check = Some(crate::now_ms());
        self.update.manual |= manual;
        if manual {
            self.status(tl!("Checking for updates…"));
        }
        true
    }

    /// Each frame: start the periodic checks and take in answers.
    pub(crate) fn update_tick(&mut self, now: f64) {
        let started = *self.update.started_at.get_or_insert(now);
        let due = match self.update.last_check {
            None => now - started >= FIRST_CHECK_AFTER_MS,
            Some(at) => now - at >= CHECK_EVERY_MS,
        };
        if due && self.services.check_update.is_some() {
            self.check_for_update(false);
            self.update.last_check = Some(now);
        }
        let Some(slot) = self.services.update_slot.clone() else { return };
        let answer = std::mem::take(&mut *slot.lock().unwrap_or_else(|e| e.into_inner()));
        let UpdateResult::Done(answer) = answer else { return };
        let manual = std::mem::take(&mut self.update.manual);
        match answer {
            Ok(info) if is_newer(&info.version, env!("CARGO_PKG_VERSION")) => {
                if manual || self.update.dismissed.as_deref() != Some(info.version.as_str()) {
                    self.update.available = Some(info);
                }
            }
            Ok(_) => {
                self.update.available = None;
                if manual {
                    self.status(crate::i18n::fmt(tl!("Wordpadx {version} is the latest version."), &[("version", env!("CARGO_PKG_VERSION"))]));
                }
            }
            Err(e) => {
                log::info!("update check failed: {e}");
                if manual {
                    self.status(tl!("Couldn't check for updates. Check your internet connection and try again."));
                }
            }
        }
    }

    /// Update: download the new version, or reload into it.
    fn apply_update(&mut self, info: &UpdateInfo) {
        match &info.action {
            UpdateAction::Reload => match &self.services.reload {
                Some(reload) => reload(),
                None => self.canvas.open_url = Some(info.page.clone()),
            },
            UpdateAction::Open(url) => self.canvas.open_url = Some(url.clone()),
        }
    }
}

/// The bar under the ribbon saying a new version is out.
pub fn bar(app: &mut WordApp, ui: &mut Ui) {
    let Some(info) = app.update.available.clone() else { return };
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("update_bar")
        .frame(egui::Frame::NONE.fill(t.accent).inner_margin(egui::Margin::symmetric(12, 6)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(10.0, 4.0);
                let text = crate::i18n::fmt(tl!("Wordpadx {version} is available."), &[("version", &info.version)]);
                ui.label(egui::RichText::new(text).font(medium(13.0)).color(t.on_accent));
                let label = match info.action {
                    UpdateAction::Reload => tl!("Reload to update"),
                    UpdateAction::Open(_) => tl!("Update"),
                };
                let update = egui::Button::new(egui::RichText::new(label).font(medium(12.5)).color(t.accent_text)).fill(t.ribbon);
                if ui.add(update).clicked() {
                    app.apply_update(&info);
                }
                if ui.add(egui::Button::new(egui::RichText::new(tl!("What's new")).font(regular(12.5)).color(t.on_accent)).frame(false)).clicked() {
                    app.canvas.open_url = Some(info.page.clone());
                }
                if ui.add(egui::Button::new(egui::RichText::new(tl!("Later")).font(regular(12.5)).color(t.on_accent)).frame(false)).clicked() {
                    app.update.dismissed = Some(info.version.clone());
                    app.update.available = None;
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_json_gives_the_download_for_each_platform() {
        let json = r#"{"tag_name":"v0.6.2","html_url":"https://github.com/o/r/releases/tag/v0.6.2",
            "assets":[{"name":"Wordpadx-iPad.zip","browser_download_url":"https://github.com/o/r/releases/download/v0.6.2/Wordpadx-iPad.zip"},
                      {"name":"app-debug.apk","browser_download_url":"https://github.com/o/r/releases/download/v0.6.2/app-debug.apk"}]}"#;
        let web = release_from_json(json, Platform::Web).unwrap();
        assert_eq!(web.version, "0.6.2");
        assert_eq!(web.action, UpdateAction::Reload);
        let android = release_from_json(json, Platform::Android).unwrap();
        assert_eq!(android.action, UpdateAction::Open("https://github.com/o/r/releases/download/v0.6.2/app-debug.apk".into()));
        let ios = release_from_json(json, Platform::Ios).unwrap();
        assert_eq!(ios.action, UpdateAction::Open("https://github.com/o/r/releases/tag/v0.6.2".into()));
        // No APK: the release page. Junk and non-https links are refused.
        let bare = r#"{"tag_name":"v1.0.0","html_url":"https://github.com/o/r/releases/tag/v1.0.0","assets":[]}"#;
        assert_eq!(
            release_from_json(bare, Platform::Android).unwrap().action,
            UpdateAction::Open("https://github.com/o/r/releases/tag/v1.0.0".into())
        );
        assert!(release_from_json("{}", Platform::Web).is_err());
        assert!(release_from_json("not json", Platform::Web).is_err());
        assert!(release_from_json(r#"{"tag_name":"v1","html_url":"javascript:alert(1)"}"#, Platform::Web).is_err());
        let evil = r#"{"tag_name":"v9.0.0","html_url":"https://x/r","assets":[{"name":"a.apk","browser_download_url":"http://evil/a.apk"}]}"#;
        assert_eq!(release_from_json(evil, Platform::Android).unwrap().action, UpdateAction::Open("https://x/r".into()));
    }

    #[test]
    fn versions_compare_numerically() {
        assert!(is_newer("v0.6.2", "0.6.1"));
        assert!(is_newer("0.10.0", "0.9.9"));
        assert!(is_newer("1.0", "0.99.99"));
        assert!(is_newer("0.7.0", "0.7.0-rc.1"), "a release is newer than its pre-release");
        assert!(!is_newer("0.6.1", "0.6.1"));
        assert!(!is_newer("0.6.0", "0.6.1"));
        assert!(!is_newer("v0.6.1-build.20", "0.6.1"));
        for junk in ["", "latest", "v", "1.2.3.4", "1.x.0", "nightly"] {
            assert!(!is_newer(junk, "0.6.1"), "{junk:?}");
        }
    }

    fn app_with_updates(answer: Result<UpdateInfo, String>) -> (WordApp, UpdateSlot, std::rc::Rc<std::cell::Cell<u32>>) {
        let slot = UpdateSlot::default();
        let asked = std::rc::Rc::new(std::cell::Cell::new(0u32));
        let services = crate::Services {
            check_update: Some(Box::new({
                let (slot, asked, answer) = (slot.clone(), asked.clone(), answer.clone());
                move || {
                    asked.set(asked.get() + 1);
                    *slot.lock().unwrap() = UpdateResult::Done(answer.clone());
                }
            })),
            update_slot: Some(slot.clone()),
            ..Default::default()
        };
        (WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), services), slot, asked)
    }

    fn newer() -> UpdateInfo {
        UpdateInfo {
            version: "99.0.0".into(),
            action: UpdateAction::Open("https://example.org/app.apk".into()),
            page: "https://example.org/r".into(),
        }
    }

    #[test]
    fn a_newer_release_shows_the_bar_once_and_later_hides_it() {
        let (mut a, _slot, asked) = app_with_updates(Ok(newer()));
        a.update_tick(0.0);
        assert_eq!(asked.get(), 0, "not while the app opens");
        a.update_tick(FIRST_CHECK_AFTER_MS + 1.0);
        assert_eq!(asked.get(), 1);
        assert_eq!(a.update.available.as_ref().map(|i| i.version.as_str()), Some("99.0.0"));
        // Later: gone, and the periodic check doesn't bring the same version back.
        a.update.dismissed = Some("99.0.0".into());
        a.update.available = None;
        a.update_tick(FIRST_CHECK_AFTER_MS + 2.0);
        assert_eq!(asked.get(), 1, "one check per period");
        a.update_tick(FIRST_CHECK_AFTER_MS + CHECK_EVERY_MS + 2.0);
        assert_eq!(asked.get(), 2);
        assert!(a.update.available.is_none(), "dismissed this session");
        // Asking from Help shows it again.
        a.check_for_update(true);
        a.update_tick(FIRST_CHECK_AFTER_MS + CHECK_EVERY_MS + 3.0);
        assert!(a.update.available.is_some());
        // Update opens the download.
        let info = a.update.available.clone().unwrap();
        a.apply_update(&info);
        assert_eq!(a.canvas.open_url.as_deref(), Some("https://example.org/app.apk"));
    }

    #[test]
    fn up_to_date_and_failed_checks_only_speak_when_asked() {
        let current = UpdateInfo { version: env!("CARGO_PKG_VERSION").into(), ..newer() };
        let (mut a, _, _) = app_with_updates(Ok(current));
        a.update_tick(0.0);
        a.update_tick(FIRST_CHECK_AFTER_MS + 1.0);
        assert!(a.update.available.is_none());
        assert!(a.status_msg.is_none(), "no news, no message");
        a.check_for_update(true);
        a.update_tick(FIRST_CHECK_AFTER_MS + 2.0);
        assert!(a.status_msg.as_ref().is_some_and(|(m, _)| m.contains("latest")), "{:?}", a.status_msg);
        let (mut a, _, _) = app_with_updates(Err("offline".into()));
        a.update_tick(0.0);
        a.update_tick(FIRST_CHECK_AFTER_MS + 1.0);
        assert!(a.status_msg.is_none());
        a.check_for_update(true);
        a.update_tick(FIRST_CHECK_AFTER_MS + 2.0);
        assert!(a.status_msg.as_ref().is_some_and(|(m, _)| m.contains("Couldn't")));
    }

    #[test]
    fn without_a_host_check_nothing_happens_and_asking_says_so() {
        let mut a = WordApp::new(wordcraft_engine::Session::new(wordcraft_doc::Document::new()), crate::Services::default());
        a.update_tick(FIRST_CHECK_AFTER_MS * 2.0);
        assert!(a.update.available.is_none());
        assert!(!a.check_for_update(true));
        assert!(a.status_msg.is_some());
    }
}
