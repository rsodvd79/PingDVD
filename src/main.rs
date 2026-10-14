use std::{
    collections::VecDeque,
    fs,
    io::ErrorKind,
    net::IpAddr,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use eframe::egui::{self, Color32, Pos2, Stroke, StrokeKind, Vec2};
use serde::{Deserialize, Serialize};
use tokio::{process::Command, sync::watch, time::sleep};

const DEFAULT_HOST: &str = "www.google.it";
const DEFAULT_INTERVAL_MS: f64 = 500.0;
const DEFAULT_TIMEOUT_MS: f64 = 500.0;
const MAX_HISTORY_SIZE: usize = 500;
const MAX_SETTING_MS: f64 = 100_000.0;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
struct Settings {
    #[serde(alias = "Host")]
    host: String,
    #[serde(alias = "Interval")]
    interval: f64,
    #[serde(alias = "TimeOut", alias = "Timeout")]
    timeout: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_owned(),
            interval: DEFAULT_INTERVAL_MS,
            timeout: DEFAULT_TIMEOUT_MS,
        }
    }
}

enum PingOutcome {
    Reply(u64),
    Timeout(u64),
    Error(String),
}

struct PingResult {
    outcome: PingOutcome,
}

struct PingApp {
    settings: Settings,
    history: VecDeque<u64>,
    status: String,
    status_color: Color32,
    running: bool,
    started_at: Option<Instant>,
    worker_settings: Option<watch::Sender<Settings>>,
    worker_results: Option<Receiver<PingResult>>,
    worker_thread: Option<JoinHandle<()>>,
    last_worker_settings: Option<Settings>,
}

impl PingApp {
    fn new() -> Self {
        let (settings, warning) = match load_settings() {
            Ok(settings) => (settings, None),
            Err(error) => (Settings::default(), Some(error)),
        };
        let mut app = Self {
            settings,
            history: initial_samples(),
            status: "Stopped".to_owned(),
            status_color: Color32::RED,
            running: false,
            started_at: None,
            worker_settings: None,
            worker_results: None,
            worker_thread: None,
            last_worker_settings: None,
        };
        if let Some(warning) = warning {
            app.status = warning;
            app.status_color = Color32::from_rgb(255, 165, 0);
        }
        app
    }

    fn start(&mut self) {
        self.settings.host = self.settings.host.trim().to_owned();
        self.running = true;
        if !valid_host(&self.settings.host) {
            self.settings.host = DEFAULT_HOST.to_owned();
            self.status = format!("Invalid host; using {DEFAULT_HOST}");
            self.status_color = Color32::from_rgb(255, 165, 0);
        } else {
            self.status = "Running".to_owned();
            self.status_color = Color32::GREEN;
        }

        self.history.clear();
        self.started_at = Some(Instant::now());
        let (settings_tx, settings_rx) = watch::channel(self.settings.clone());
        let (results_tx, results_rx) = mpsc::channel();
        self.worker_thread = Some(thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = results_tx.send(PingResult {
                        outcome: PingOutcome::Error(format!(
                            "Could not start ping worker: {error}"
                        )),
                    });
                    return;
                }
            };
            runtime.block_on(run_ping_loop(settings_rx, results_tx));
        }));
        self.worker_settings = Some(settings_tx);
        self.worker_results = Some(results_rx);
        self.last_worker_settings = Some(self.settings.clone());
    }

    fn stop(&mut self) {
        self.running = false;
        self.worker_settings = None;
        if let Some(worker) = self.worker_thread.take() {
            if worker.join().is_err() {
                self.status = "Ping worker stopped unexpectedly".to_owned();
                self.status_color = Color32::from_rgb(255, 165, 0);
            }
        }
        self.worker_results = None;
        self.last_worker_settings = None;
        if self.status == "Running" || self.status.starts_with("Running -") {
            self.status = "Stopped".to_owned();
            self.status_color = Color32::RED;
        }
    }

    fn update_worker_settings(&mut self) {
        if self.last_worker_settings.as_ref() == Some(&self.settings) {
            return;
        }
        if let Some(sender) = &self.worker_settings
            && sender.send(self.settings.clone()).is_err()
        {
            self.status = "Ping worker is no longer available".to_owned();
            self.status_color = Color32::from_rgb(255, 165, 0);
        }
        self.last_worker_settings = Some(self.settings.clone());
    }

    fn poll_worker(&mut self) {
        let results = self
            .worker_results
            .as_ref()
            .map(Receiver::try_iter)
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        for result in results {
            match result.outcome {
                PingOutcome::Reply(milliseconds) => {
                    self.push_sample(milliseconds);
                    self.status = format!("Running - Last: {milliseconds} ms");
                    self.status_color = Color32::GREEN;
                }
                PingOutcome::Timeout(milliseconds) => {
                    self.push_sample(milliseconds);
                    self.status = format!("Running - Timeout: {milliseconds} ms");
                    self.status_color = Color32::from_rgb(255, 165, 0);
                }
                PingOutcome::Error(error) => {
                    self.status = format!("Error: {error}");
                    self.status_color = Color32::from_rgb(255, 165, 0);
                }
            }
        }
    }

    fn push_sample(&mut self, milliseconds: u64) {
        self.history.push_back(milliseconds);
        if self.history.len() > MAX_HISTORY_SIZE {
            self.history.pop_front();
        }
    }

    fn save_settings(&mut self) {
        match write_settings(&self.settings) {
            Ok(()) => {
                self.status = "Settings saved".to_owned();
                self.status_color = Color32::GREEN;
            }
            Err(error) => {
                self.status = format!("Could not save settings: {error}");
                self.status_color = Color32::from_rgb(255, 165, 0);
            }
        }
    }

    fn reset_settings(&mut self) {
        self.settings = Settings::default();
        self.update_worker_settings();
        self.save_settings();
    }

    fn elapsed(&self) -> String {
        let seconds = self.started_at.map_or(0, |start| start.elapsed().as_secs());
        format!(
            "{:02}:{:02}:{:02}",
            seconds / 3600,
            (seconds / 60) % 60,
            seconds % 60
        )
    }

    fn draw_chart(&self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
        let rect = response.rect.shrink(8.0);
        painter.rect_filled(
            rect,
            egui::CornerRadius::same(3),
            Color32::from_rgb(17, 17, 17),
        );
        painter.rect_stroke(
            rect,
            egui::CornerRadius::same(3),
            Stroke::new(1.0_f32, Color32::from_gray(85)),
            StrokeKind::Outside,
        );

        if self.history.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Waiting for ping samples...",
                egui::FontId::proportional(14.0),
                Color32::GRAY,
            );
            return;
        }

        let minimum = *self.history.iter().min().unwrap_or(&0) as f64;
        let maximum = *self.history.iter().max().unwrap_or(&0) as f64;
        let actual_range = maximum - minimum;
        let range = actual_range.max(10.0);
        let range_min = if actual_range < 10.0 {
            (minimum + maximum) / 2.0 - 5.0
        } else {
            minimum
        };
        let x_step = rect.width() / self.history.len().saturating_sub(1).max(1) as f32;
        let points: Vec<Pos2> = self
            .history
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let x = rect.left() + index as f32 * x_step;
                let y = rect.bottom()
                    - ((*value as f64 - range_min) / range * rect.height() as f64) as f32;
                Pos2::new(x, y)
            })
            .collect();
        for pair in points.windows(2) {
            painter.line_segment(
                [pair[0], pair[1]],
                Stroke::new(2.0_f32, Color32::from_rgb(0, 191, 255)),
            );
        }
        if points.len() == 1 {
            painter.circle_filled(points[0], 2.0, Color32::from_rgb(0, 191, 255));
        }

        let average =
            self.history.iter().map(|value| *value as f64).sum::<f64>() / self.history.len() as f64;
        let average_y =
            rect.bottom() - ((average - range_min) / range * rect.height() as f64) as f32;
        let mut x = rect.left();
        while x < rect.right() {
            let end = (x + 4.0).min(rect.right());
            painter.line_segment(
                [Pos2::new(x, average_y), Pos2::new(end, average_y)],
                Stroke::new(1.0_f32, Color32::from_rgb(255, 69, 0)),
            );
            x += 8.0;
        }
    }
}

impl eframe::App for PingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_worker();
        let mut settings_changed = false;
        let mut start_clicked = false;
        let mut stop_clicked = false;
        let mut apply_clicked = false;
        let mut reset_clicked = false;

        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Host");
                settings_changed |= ui
                    .add(egui::TextEdit::singleline(&mut self.settings.host).desired_width(150.0))
                    .changed();
                ui.label("Timeout");
                settings_changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.settings.timeout)
                            .range(1.0..=MAX_SETTING_MS)
                            .suffix(" ms"),
                    )
                    .changed();
                ui.label("Interval");
                settings_changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.settings.interval)
                            .range(1.0..=MAX_SETTING_MS)
                            .suffix(" ms"),
                    )
                    .changed();
                if ui
                    .button(if self.running {
                        "■ Stop"
                    } else {
                        "▶ Start"
                    })
                    .clicked()
                {
                    if self.running {
                        stop_clicked = true;
                    } else {
                        start_clicked = true;
                    }
                }
                apply_clicked = ui.button("Apply").clicked();
                reset_clicked = ui.button("Reset").clicked();
            });

            if !self.history.is_empty() {
                let average = self.history.iter().map(|value| *value as f64).sum::<f64>()
                    / self.history.len() as f64;
                ui.horizontal(|ui| {
                    ui.label(format!("AVG: {average:.2} ms"));
                    ui.label(format!("LAST: {} ms", self.history.back().unwrap_or(&0)));
                    ui.label(format!(
                        "MIN: {} ms",
                        self.history.iter().min().unwrap_or(&0)
                    ));
                    ui.label(format!(
                        "MAX: {} ms",
                        self.history.iter().max().unwrap_or(&0)
                    ));
                    ui.label(self.elapsed());
                });
            }
        });

        if settings_changed {
            self.update_worker_settings();
        }
        if reset_clicked {
            self.reset_settings();
        } else if apply_clicked {
            self.save_settings();
        }
        if stop_clicked {
            self.stop();
        } else if start_clicked {
            self.start();
        }

        egui::CentralPanel::default().show(ctx, |ui| self.draw_chart(ui));
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(self.status_color, "●");
                ui.label(&self.status);
            });
        });
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

impl Drop for PingApp {
    fn drop(&mut self) {
        self.stop();
        if let Err(error) = write_settings(&self.settings) {
            eprintln!("Could not save PingDVD settings: {error}");
        }
    }
}

fn initial_samples() -> VecDeque<u64> {
    (0..200)
        .map(|index| {
            let base = 12.0 + 3.0 * (index as f64 / 8.0).sin();
            let noise = (f64::from((index * 7919 % 101) as u32) / 50.0) - 1.0;
            (base + noise).round().max(0.0) as u64
        })
        .collect()
}

fn valid_host(host: &str) -> bool {
    if host.is_empty() || host.len() > 255 {
        return false;
    }
    if host.parse::<IpAddr>().is_ok() {
        return true;
    }
    host.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label.chars().next().is_some_and(char::is_alphanumeric)
            && label.chars().last().is_some_and(char::is_alphanumeric)
            && label
                .chars()
                .all(|character| character.is_alphanumeric() || character == '-')
    })
}

async fn run_ping_loop(mut settings_rx: watch::Receiver<Settings>, results_tx: Sender<PingResult>) {
    loop {
        let settings = settings_rx.borrow_and_update().clone();
        let timeout_ms = settings.timeout.clamp(1.0, MAX_SETTING_MS).round() as u64;
        let ping = ping_once(&settings.host, timeout_ms);
        let outcome = tokio::select! {
            changed = settings_rx.changed() => {
                if changed.is_err() {
                    break;
                }
                continue;
            }
            outcome = ping => outcome,
        };
        if results_tx.send(PingResult { outcome }).is_err() {
            break;
        }

        let interval_ms = settings.interval.clamp(1.0, MAX_SETTING_MS).round() as u64;
        tokio::select! {
            _ = sleep(Duration::from_millis(interval_ms)) => {}
            changed = settings_rx.changed() => {
                if changed.is_err() {
                    break;
                }
            }
        }
    }
}

async fn ping_once(host: &str, timeout_ms: u64) -> PingOutcome {
    let mut command = Command::new("ping");
    #[cfg(windows)]
    command.args(["-n", "1"]);
    #[cfg(target_os = "macos")]
    command.args(["-n", "-c", "1"]);
    #[cfg(all(unix, not(target_os = "macos")))]
    command.args(["-n", "-c", "1"]);
    #[cfg(not(any(unix, windows)))]
    return PingOutcome::Error("Ping is not supported on this platform".to_owned());

    let output = match tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        command.arg(host).kill_on_drop(true).output(),
    )
    .await
    {
        Ok(Ok(output)) => output,
        Err(_) => return PingOutcome::Timeout(timeout_ms),
        Ok(Err(error)) => return PingOutcome::Error(format!("Could not run ping: {error}")),
    };
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if let Some(milliseconds) = parse_latency(&text) {
        return PingOutcome::Reply(milliseconds);
    }

    let lower = text.to_ascii_lowercase();
    if lower.contains("unreachable") || lower.contains("prohibited") {
        return PingOutcome::Error("Host unreachable".to_owned());
    }
    if lower.contains("permission denied") || lower.contains("access denied") {
        return PingOutcome::Error("Access denied".to_owned());
    }
    if lower.contains("unknown host")
        || lower.contains("could not find host")
        || lower.contains("name or service not known")
        || lower.contains("temporary failure in name resolution")
    {
        return PingOutcome::Error(format!("Could not resolve host: {host}"));
    }
    if output.status.success() {
        PingOutcome::Error(format!("No ping result received for {host}"))
    } else {
        PingOutcome::Timeout(timeout_ms)
    }
}

fn parse_latency(output: &str) -> Option<u64> {
    for token in output.split_whitespace() {
        let token = token.trim_matches(|character: char| matches!(character, ',' | ';' | ')'));
        let lower = token.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("time<") {
            if value.trim_end_matches("ms").parse::<f64>().is_ok() {
                return Some(0);
            }
        }
        if let Some(value) = lower.strip_prefix("time=") {
            let value = value.trim_end_matches("ms");
            if let Ok(milliseconds) = value.parse::<f64>() {
                return Some(milliseconds.round().max(0.0) as u64);
            }
        }
    }
    None
}

fn primary_settings_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.parent()
                .map(|parent| parent.join("pingdvd.settings.json"))
        })
        .unwrap_or_else(|| PathBuf::from("pingdvd.settings.json"))
}

fn fallback_settings_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)?
        .join("Library/Application Support");
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from)?;
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("PingDVD").join("pingdvd.settings.json"))
}

fn load_settings() -> Result<Settings, String> {
    let mut paths = vec![primary_settings_path()];
    if let Ok(current_dir) = std::env::current_dir() {
        paths.push(current_dir.join("pingdvd.settings.json"));
    }
    if let Some(fallback) = fallback_settings_path() {
        paths.push(fallback);
    }
    for path in paths {
        match fs::read_to_string(path) {
            Ok(contents) => {
                let mut settings: Settings = serde_json::from_str(&contents)
                    .map_err(|error| format!("Invalid settings file: {error}"))?;
                settings.interval = settings.interval.clamp(1.0, MAX_SETTING_MS);
                settings.timeout = settings.timeout.clamp(1.0, MAX_SETTING_MS);
                return Ok(settings);
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(format!("Could not read settings: {error}")),
        }
    }
    Ok(Settings::default())
}

fn write_settings(settings: &Settings) -> Result<(), String> {
    let contents = serde_json::to_vec_pretty(settings)
        .map_err(|error| format!("Could not encode settings: {error}"))?;
    let primary = primary_settings_path();
    match write_settings_file(&primary, &contents) {
        Ok(()) => Ok(()),
        Err(primary_error) => {
            let Some(fallback) = fallback_settings_path() else {
                return Err(format!("{} ({})", primary_error, primary.display()));
            };
            write_settings_file(&fallback, &contents).map_err(|fallback_error| {
                format!("{primary_error}; fallback failed: {fallback_error}")
            })
        }
    }
}

fn write_settings_file(path: &Path, contents: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, contents).map_err(|error| error.to_string())
}

fn app_icon() -> egui::IconData {
    let image = image::load_from_memory(include_bytes!("../assets/pingdvd.png"))
        .expect("embedded PingDVD icon is valid")
        .into_rgba8();
    let (width, height) = image.dimensions();
    egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::{initial_samples, parse_latency, valid_host};
    use serde_json::from_str;

    #[test]
    fn validates_hosts_parses_ping_output_and_loads_legacy_settings() {
        assert!(valid_host("www.google.it"));
        assert!(valid_host("127.0.0.1"));
        assert!(valid_host("::1"));
        assert!(!valid_host(""));
        assert!(!valid_host("-invalid.example"));
        assert!(!valid_host("bad..example"));

        assert_eq!(parse_latency("time=12.4 ms"), Some(12));
        assert_eq!(parse_latency("Reply from 127.0.0.1: time=7ms"), Some(7));
        assert_eq!(parse_latency("Reply from ::1: time<1ms"), Some(0));
        assert_eq!(parse_latency("Request timed out."), None);
        assert_eq!(initial_samples().len(), 200);
        let legacy =
            from_str::<super::Settings>(r#"{"Host":"example.com","Interval":125,"TimeOut":250}"#)
                .expect("legacy settings should remain readable");
        assert_eq!(legacy.host, "example.com");
        assert_eq!(legacy.interval, 125.0);
        assert_eq!(legacy.timeout, 250.0);
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(800.0, 450.0))
            .with_icon(app_icon()),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "PingDVD",
        options,
        Box::new(|_creation_context| Ok(Box::new(PingApp::new()))),
    )
}
