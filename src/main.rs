use std::{
    collections::VecDeque,
    fs::{self, OpenOptions},
    io::{ErrorKind, Write},
    net::IpAddr,
    path::{Path, PathBuf},
    process::{Output, Stdio},
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Command,
    sync::watch,
    time::sleep,
};

const DEFAULT_HOST: &str = "www.google.it";
const DEFAULT_INTERVAL_MS: f64 = 500.0;
const DEFAULT_TIMEOUT_MS: f64 = 500.0;
const MAX_HISTORY_SIZE: usize = 500;
const MAX_SETTING_MS: f64 = 100_000.0;
const BACKGROUND: Color32 = Color32::from_rgb(15, 20, 29);
const SURFACE: Color32 = Color32::from_rgb(23, 31, 43);
const ACCENT: Color32 = Color32::from_rgb(72, 207, 238);
const MUTED: Color32 = Color32::from_rgb(164, 179, 199);
const SUCCESS: Color32 = Color32::from_rgb(102, 222, 164);
const WARNING: Color32 = Color32::from_rgb(255, 195, 106);
const ERROR: Color32 = Color32::from_rgb(255, 133, 143);

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

impl Settings {
    fn normalized(&self) -> Result<Self, String> {
        let host = self.host.trim();
        if !valid_host(host) {
            return Err("Enter a hostname or IP address, without a URL or port.".to_owned());
        }
        if !self.interval.is_finite() || !self.timeout.is_finite() {
            return Err("Interval and timeout must be finite numbers.".to_owned());
        }
        Ok(Self {
            host: host
                .parse::<IpAddr>()
                .map_or_else(|_| host.to_ascii_lowercase(), |address| address.to_string()),
            interval: self.interval.clamp(1.0, MAX_SETTING_MS),
            timeout: self.timeout.clamp(1.0, MAX_SETTING_MS),
        })
    }
}

#[derive(Debug, PartialEq)]
enum PingOutcome {
    Reply(u64),
    Timeout(u64),
    Error(String),
}

struct PingResult {
    session_id: u64,
    outcome: PingOutcome,
}

#[derive(Clone)]
struct WorkerSettings {
    settings: Settings,
    session_id: u64,
}

#[derive(Debug, PartialEq)]
struct LatencyStats {
    average: f64,
    minimum: u64,
    maximum: u64,
}

fn latency_stats(history: &VecDeque<PingOutcome>) -> Option<LatencyStats> {
    let mut replies = history.iter().filter_map(|outcome| match outcome {
        PingOutcome::Reply(value) => Some(*value),
        _ => None,
    });
    let first = replies.next()?;
    let (sum, count, minimum, maximum) = replies.fold(
        (first as f64, 1, first, first),
        |(sum, count, minimum, maximum), value| {
            (
                sum + value as f64,
                count + 1,
                minimum.min(value),
                maximum.max(value),
            )
        },
    );
    Some(LatencyStats {
        average: sum / f64::from(count),
        minimum,
        maximum,
    })
}

struct Notice {
    text: String,
    color: Color32,
    expires_at: Option<Instant>,
}

struct PingApp {
    settings: Settings,
    history: VecDeque<PingOutcome>,
    running: bool,
    session_id: u64,
    started_at: Option<Instant>,
    stopped_elapsed: Duration,
    worker_settings: Option<watch::Sender<WorkerSettings>>,
    worker_results: Option<Receiver<PingResult>>,
    worker_thread: Option<JoinHandle<()>>,
    last_worker_settings: Option<Settings>,
    notice: Option<Notice>,
    autosave: bool,
}

impl PingApp {
    fn new() -> Self {
        let (settings, warning) = load_settings_from(&settings_load_paths());
        let autosave = warning.is_none();
        Self {
            settings,
            history: VecDeque::new(),
            running: false,
            session_id: 0,
            started_at: None,
            stopped_elapsed: Duration::ZERO,
            worker_settings: None,
            worker_results: None,
            worker_thread: None,
            last_worker_settings: None,
            notice: warning.map(|text| Notice {
                text,
                color: WARNING,
                expires_at: None,
            }),
            autosave,
        }
    }

    fn show_notice(&mut self, text: String, color: Color32) {
        self.notice = Some(Notice {
            text,
            color,
            expires_at: Some(Instant::now() + Duration::from_secs(6)),
        });
    }

    fn reset_session(&mut self) {
        self.session_id = self.session_id.wrapping_add(1);
        self.history.clear();
        self.started_at = Some(Instant::now());
        self.stopped_elapsed = Duration::ZERO;
    }

    fn start(&mut self) {
        if self.running {
            return;
        }
        let settings = match self.settings.normalized() {
            Ok(settings) => settings,
            Err(error) => {
                self.show_notice(error, ERROR);
                return;
            }
        };
        self.settings = settings.clone();
        self.reset_session();
        self.running = true;
        let initial = WorkerSettings {
            settings: settings.clone(),
            session_id: self.session_id,
        };
        let (settings_tx, settings_rx) = watch::channel(initial);
        let (results_tx, results_rx) = mpsc::channel();
        let session_id = self.session_id;
        self.worker_thread = Some(thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = results_tx.send(PingResult {
                        session_id,
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
        self.last_worker_settings = Some(settings);
    }

    fn stop(&mut self) {
        if let Some(started_at) = self.started_at.take() {
            self.stopped_elapsed = started_at.elapsed();
        }
        self.running = false;
        self.worker_settings = None;
        if let Some(worker) = self.worker_thread.take()
            && worker.join().is_err()
        {
            self.show_notice("Ping worker stopped unexpectedly.".to_owned(), ERROR);
        }
        self.worker_results = None;
    }

    fn update_worker_settings(&mut self) {
        let Ok(settings) = self.settings.normalized() else {
            return;
        };
        if !self.running || self.last_worker_settings.as_ref() == Some(&settings) {
            return;
        }
        if self
            .last_worker_settings
            .as_ref()
            .is_some_and(|active| active.host != settings.host)
        {
            self.reset_session();
        }
        let updated = WorkerSettings {
            settings: settings.clone(),
            session_id: self.session_id,
        };
        if self
            .worker_settings
            .as_ref()
            .is_none_or(|sender| sender.send(updated).is_err())
        {
            self.stop();
            self.show_notice("Ping worker is no longer available.".to_owned(), ERROR);
            return;
        }
        self.last_worker_settings = Some(settings);
    }

    fn poll_worker(&mut self) {
        loop {
            let result = match self.worker_results.as_ref().map(Receiver::try_recv) {
                Some(Ok(result)) => result,
                Some(Err(TryRecvError::Disconnected)) => {
                    self.stop();
                    self.show_notice(
                        "Ping worker disconnected. You can start again.".to_owned(),
                        ERROR,
                    );
                    break;
                }
                _ => break,
            };
            if self.running && result.session_id == self.session_id {
                self.push_sample(result.outcome);
            }
        }
    }

    fn push_sample(&mut self, outcome: PingOutcome) {
        self.history.push_back(outcome);
        if self.history.len() > MAX_HISTORY_SIZE {
            self.history.pop_front();
        }
    }

    fn save_settings(&mut self) {
        match self.settings.normalized().and_then(|settings| {
            write_settings_to(&settings, &settings_save_paths())?;
            Ok(settings)
        }) {
            Ok(settings) => {
                self.settings = settings;
                self.autosave = true;
                self.show_notice("Settings saved.".to_owned(), SUCCESS);
            }
            Err(error) => self.show_notice(format!("Could not save settings: {error}"), ERROR),
        }
    }

    fn reset_settings(&mut self) {
        self.settings = Settings::default();
        self.update_worker_settings();
        self.save_settings();
    }

    fn elapsed_duration(&self) -> Duration {
        self.started_at
            .map_or(self.stopped_elapsed, |start| start.elapsed())
    }

    fn elapsed(&self) -> String {
        let seconds = self.elapsed_duration().as_secs();
        format!(
            "{:02}:{:02}:{:02}",
            seconds / 3600,
            (seconds / 60) % 60,
            seconds % 60
        )
    }

    fn status(&self) -> (String, Color32) {
        if !self.running {
            return ("Stopped".to_owned(), MUTED);
        }
        match self.history.back() {
            Some(PingOutcome::Reply(ms)) => (format!("Running - Last reply: {ms} ms"), SUCCESS),
            Some(PingOutcome::Timeout(ms)) => (format!("Running - Timeout after {ms} ms"), WARNING),
            Some(PingOutcome::Error(error)) => (format!("Running - {error}"), ERROR),
            None => ("Running - Waiting for a reply...".to_owned(), ACCENT),
        }
    }

    fn draw_chart(&self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
        let outer = response.rect;
        painter.rect_filled(outer, 8, BACKGROUND);
        if outer.width() < 150.0 || outer.height() < 90.0 {
            painter.text(
                outer.center(),
                egui::Align2::CENTER_CENTER,
                "Enlarge the window to see the chart",
                egui::FontId::proportional(12.0),
                MUTED,
            );
            return;
        }
        let rect = egui::Rect::from_min_max(
            outer.min + Vec2::new(54.0, 20.0),
            outer.max - Vec2::new(18.0, 30.0),
        );
        let stats = latency_stats(&self.history);
        let (minimum, maximum) = stats.as_ref().map_or((0.0, 10.0), |stats| {
            let padding = ((stats.maximum - stats.minimum) as f64 * 0.15).max(5.0);
            (
                (stats.minimum as f64 - padding).max(0.0),
                stats.maximum as f64 + padding,
            )
        });
        let y_for = |value: f64| {
            rect.bottom() - ((value - minimum) / (maximum - minimum)) as f32 * rect.height()
        };
        let ticks = if rect.height() < 100.0 { 2 } else { 4 };
        for index in 0..=ticks {
            let value = minimum + (maximum - minimum) * f64::from(index) / f64::from(ticks);
            let y = y_for(value);
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0_f32, Color32::from_rgb(40, 52, 69)),
            );
            painter.text(
                Pos2::new(rect.left() - 9.0, y),
                egui::Align2::RIGHT_CENTER,
                format!("{value:.0}"),
                egui::FontId::monospace(11.0),
                MUTED,
            );
        }
        painter.text(
            Pos2::new(outer.left() + 16.0, outer.top() + 8.0),
            egui::Align2::LEFT_TOP,
            "ms",
            egui::FontId::proportional(11.0),
            MUTED,
        );
        painter.text(
            Pos2::new(rect.center().x, outer.bottom() - 9.0),
            egui::Align2::CENTER_BOTTOM,
            "Recent attempts (oldest -> newest)",
            egui::FontId::proportional(11.0),
            MUTED,
        );
        if self.history.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No measurements yet. Start a ping session.",
                egui::FontId::proportional(14.0),
                MUTED,
            );
            return;
        }
        let x_step = rect.width() / self.history.len().saturating_sub(1).max(1) as f32;
        let mut previous = None;
        for (index, outcome) in self.history.iter().enumerate() {
            let x = if self.history.len() == 1 {
                rect.center().x
            } else {
                rect.left() + index as f32 * x_step
            };
            match outcome {
                PingOutcome::Reply(value) => {
                    let point = Pos2::new(x, y_for(*value as f64));
                    if let Some(previous) = previous {
                        painter.line_segment([previous, point], Stroke::new(2.0_f32, ACCENT));
                    }
                    painter.circle_filled(point, 2.0, ACCENT);
                    previous = Some(point);
                }
                _ => {
                    previous = None;
                    let color = if matches!(outcome, PingOutcome::Timeout(_)) {
                        WARNING
                    } else {
                        ERROR
                    };
                    painter.line_segment(
                        [
                            Pos2::new(x, rect.bottom() - 6.0),
                            Pos2::new(x, rect.bottom()),
                        ],
                        Stroke::new(2.0_f32, color),
                    );
                }
            }
        }
        if let Some(stats) = stats {
            let y = y_for(stats.average);
            let mut x = rect.left();
            while x < rect.right() {
                painter.line_segment(
                    [Pos2::new(x, y), Pos2::new((x + 4.0).min(rect.right()), y)],
                    Stroke::new(1.0_f32, SUCCESS),
                );
                x += 8.0;
            }
        }
    }
}

fn stat_card(ui: &mut egui::Ui, compact: bool, label: &str, value: String) {
    if compact {
        ui.label(
            egui::RichText::new(format!("{label} {value}"))
                .monospace()
                .size(12.0),
        );
        return;
    }
    egui::Frame::new()
        .fill(BACKGROUND)
        .corner_radius(6)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_min_width(90.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(label).size(10.0).color(MUTED));
                ui.label(egui::RichText::new(value).monospace().size(17.0));
            });
        });
}

fn timing_control(ui: &mut egui::Ui, label: &str, value: &mut f64, help: &str) -> bool {
    ui.label(label);
    ui.add(
        egui::DragValue::new(value)
            .range(1.0..=MAX_SETTING_MS)
            .suffix(" ms"),
    )
    .on_hover_text(help)
    .changed()
}

impl PingApp {
    fn show(&mut self, ctx: &egui::Context) {
        self.poll_worker();
        if self
            .notice
            .as_ref()
            .and_then(|notice| notice.expires_at)
            .is_some_and(|expiry| Instant::now() >= expiry)
        {
            self.notice = None;
        }
        let mut settings_changed = false;
        let mut start_clicked = false;
        let mut stop_clicked = false;
        let mut apply_clicked = false;
        let mut reset_clicked = false;

        egui::TopBottomPanel::top("controls")
            .frame(egui::Frame::new().fill(SURFACE).inner_margin(14))
            .show(ctx, |ui| {
                let compact = ui.available_width() < 720.0;
                ui.horizontal(|ui| {
                    ui.heading("PingDVD");
                    ui.label(
                        egui::RichText::new("NETWORK LATENCY")
                            .size(10.0)
                            .color(MUTED),
                    );
                });
                ui.add_space(10.0);
                let running = self.running;
                let mut actions = |ui: &mut egui::Ui, settings: &Settings| {
                    let button = egui::Button::new(
                        egui::RichText::new(if running { "Stop" } else { "Start" })
                            .color(BACKGROUND)
                            .strong(),
                    )
                    .fill(if running { WARNING } else { ACCENT });
                    if ui
                        .add_enabled(running || settings.normalized().is_ok(), button)
                        .clicked()
                    {
                        if running {
                            stop_clicked = true;
                        } else {
                            start_clicked = true;
                        }
                    }
                    apply_clicked = ui
                        .add_enabled(settings.normalized().is_ok(), egui::Button::new("Apply"))
                        .on_hover_text(
                            "Save settings. Valid edits already take effect immediately.",
                        )
                        .clicked();
                    reset_clicked = ui
                        .button("Reset")
                        .on_hover_text("Restore and save the default settings.")
                        .clicked();
                };
                ui.horizontal(|ui| {
                    ui.label("Host");
                    settings_changed |= ui
                        .add(
                            egui::TextEdit::singleline(&mut self.settings.host)
                                .desired_width(if compact { 150.0 } else { 190.0 })
                                .hint_text("Hostname or IP address"),
                        )
                        .changed();
                    settings_changed |= timing_control(
                        ui,
                        "Timeout",
                        &mut self.settings.timeout,
                        "Maximum wait for each ping, including name resolution.",
                    );
                    if !compact {
                        settings_changed |= timing_control(
                            ui,
                            "Interval",
                            &mut self.settings.interval,
                            "Wait after each completed attempt, not start-to-start.",
                        );
                        actions(ui, &self.settings);
                    }
                });
                if compact {
                    ui.horizontal(|ui| {
                        settings_changed |= timing_control(
                            ui,
                            "Interval",
                            &mut self.settings.interval,
                            "Wait after each completed attempt, not start-to-start.",
                        );
                        actions(ui, &self.settings);
                    });
                }
                if let Err(error) = self.settings.normalized() {
                    ui.colored_label(ERROR, error);
                }
                if let Some(active) = &self.last_worker_settings {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} host: {}",
                            if self.running {
                                "Active"
                            } else {
                                "Last session"
                            },
                            active.host
                        ))
                        .color(MUTED)
                        .size(12.0),
                    );
                }
                ui.add_space(8.0);
                let stats = latency_stats(&self.history);
                ui.horizontal_wrapped(|ui| {
                    stat_card(
                        ui,
                        compact,
                        "AVG",
                        stats
                            .as_ref()
                            .map_or("--".to_owned(), |stats| format!("{:.2} ms", stats.average)),
                    );
                    stat_card(
                        ui,
                        compact,
                        "LAST",
                        match self.history.back() {
                            Some(PingOutcome::Reply(ms)) => format!("{ms} ms"),
                            Some(PingOutcome::Timeout(_)) => "Timeout".to_owned(),
                            Some(PingOutcome::Error(_)) => "Error".to_owned(),
                            None => "--".to_owned(),
                        },
                    );
                    stat_card(
                        ui,
                        compact,
                        "MIN",
                        stats
                            .as_ref()
                            .map_or("--".to_owned(), |stats| format!("{} ms", stats.minimum)),
                    );
                    stat_card(
                        ui,
                        compact,
                        "MAX",
                        stats
                            .as_ref()
                            .map_or("--".to_owned(), |stats| format!("{} ms", stats.maximum)),
                    );
                    stat_card(ui, compact, "SESSION", self.elapsed());
                });
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

        egui::TopBottomPanel::bottom("status")
            .frame(egui::Frame::new().fill(SURFACE).inner_margin(10))
            .show(ctx, |ui| {
                let (status, color) = self.status();
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(color, status);
                    if let Some(notice) = &self.notice {
                        ui.separator();
                        ui.colored_label(notice.color, &notice.text);
                    }
                });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BACKGROUND).inner_margin(14))
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.strong("Latency");
                    ui.label(egui::RichText::new("Last 500 attempts").color(MUTED));
                    ui.colored_label(ACCENT, "Reply");
                    ui.colored_label(SUCCESS, "Average (dashed)");
                    ui.colored_label(WARNING, "Timeout (tick)");
                    ui.colored_label(ERROR, "Error (tick)");
                });
                ui.add_space(8.0);
                self.draw_chart(ui);
            });
        if self.running {
            ctx.request_repaint_after(Duration::from_millis(50));
        } else if let Some(expiry) = self.notice.as_ref().and_then(|notice| notice.expires_at) {
            ctx.request_repaint_after(expiry.saturating_duration_since(Instant::now()));
        }
    }
}

impl eframe::App for PingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show(ctx);
    }
}

impl Drop for PingApp {
    fn drop(&mut self) {
        self.stop();
        if self.autosave
            && let Err(error) = self
                .settings
                .normalized()
                .and_then(|settings| write_settings_to(&settings, &settings_save_paths()))
        {
            eprintln!("Could not save PingDVD settings: {error}");
        }
    }
}

fn valid_host(host: &str) -> bool {
    if host.is_empty() || host.len() > 255 {
        return false;
    }
    if host.parse::<IpAddr>().is_ok() {
        return true;
    }
    host.strip_suffix('.')
        .unwrap_or(host)
        .split('.')
        .all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label.as_bytes()[0].is_ascii_alphanumeric()
                && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
                && label
                    .bytes()
                    .all(|character| character.is_ascii_alphanumeric() || character == b'-')
        })
}

async fn run_ping_loop(
    mut settings_rx: watch::Receiver<WorkerSettings>,
    results_tx: Sender<PingResult>,
) {
    loop {
        let current = settings_rx.borrow_and_update().clone();
        let timeout_ms = current.settings.timeout.round() as u64;
        let Some(outcome) = collect_ping_output(
            ping_command(&current.settings.host),
            timeout_ms,
            Some(&mut settings_rx),
        )
        .await
        else {
            if settings_rx.has_changed().is_err() {
                break;
            }
            continue;
        };
        if results_tx
            .send(PingResult {
                session_id: current.session_id,
                outcome,
            })
            .is_err()
        {
            break;
        }
        tokio::select! {
            biased;
            changed = settings_rx.changed() => {
                if changed.is_err() {
                    break;
                }
            }
            _ = sleep(Duration::from_millis(current.settings.interval.round() as u64)) => {}
        }
    }
}

fn ping_command(host: &str) -> Command {
    #[cfg(target_os = "macos")]
    let mut command = Command::new(if host.parse::<std::net::Ipv6Addr>().is_ok() {
        "ping6"
    } else {
        "ping"
    });
    #[cfg(not(target_os = "macos"))]
    let mut command = Command::new("ping");
    #[cfg(windows)]
    {
        command.args(["-n", "1"]);
        command.creation_flags(0x0800_0000); // Keep the worker's console window hidden.
    }
    #[cfg(unix)]
    {
        command.args(["-n", "-c", "1"]);
        command.env("LC_ALL", "C");
    }
    command.arg(host).kill_on_drop(true);
    command
}

#[cfg(test)]
async fn ping_once(host: &str, timeout_ms: u64) -> PingOutcome {
    collect_ping_output(ping_command(host), timeout_ms, None)
        .await
        .expect("without a settings channel the ping cannot be cancelled")
}

async fn read_pipe(pipe: Option<impl AsyncRead + Unpin>) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    if let Some(mut pipe) = pipe {
        pipe.read_to_end(&mut bytes).await?;
    }
    Ok(bytes)
}

async fn collect_ping_output(
    mut command: Command,
    timeout_ms: u64,
    mut cancel: Option<&mut watch::Receiver<WorkerSettings>>,
) -> Option<PingOutcome> {
    if let Some(receiver) = cancel.as_deref_mut()
        && receiver.has_changed().unwrap_or(true)
    {
        let _ = receiver.changed().await;
        return None;
    }
    let mut child = match command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
    {
        Ok(child) => child,
        Err(error) => return Some(PingOutcome::Error(format!("Could not run ping: {error}"))),
    };
    let stdout = read_pipe(child.stdout.take());
    let stderr = read_pipe(child.stderr.take());
    let (cancelled, output) = {
        let collect = async {
            let (status, stdout, stderr) = tokio::join!(child.wait(), stdout, stderr);
            Ok::<_, std::io::Error>(Output {
                status: status?,
                stdout: stdout?,
                stderr: stderr?,
            })
        };
        tokio::select! {
            biased;
            _ = async {
                match cancel {
                    Some(receiver) => { let _ = receiver.changed().await; }
                    None => std::future::pending::<()>().await,
                }
            } => (true, None),
            _ = sleep(Duration::from_millis(timeout_ms)) => (false, None),
            result = collect => (false, Some(result)),
        }
    };
    let output = match output {
        Some(Ok(output)) => output,
        Some(Err(error)) => {
            return Some(PingOutcome::Error(format!("Could not read ping: {error}")));
        }
        None => {
            // Reap before dropping the runtime: kill_on_drop alone can leave a zombie after Stop.
            if let Err(error) = child.kill().await {
                return Some(PingOutcome::Error(format!("Could not stop ping: {error}")));
            }
            return (!cancelled).then_some(PingOutcome::Timeout(timeout_ms));
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Some(classify_ping_output(
        output.status.success(),
        &stdout,
        &stderr,
        timeout_ms,
    ))
}

fn classify_ping_output(success: bool, stdout: &str, stderr: &str, timeout_ms: u64) -> PingOutcome {
    if success && let Some(milliseconds) = parse_latency(stdout) {
        return PingOutcome::Reply(milliseconds);
    }
    if !stderr.trim().is_empty() {
        return PingOutcome::Error(stderr.trim().to_owned());
    }
    let lower = stdout.to_ascii_lowercase();
    if lower.contains("unreachable") || lower.contains("prohibited") {
        return PingOutcome::Error("Host unreachable".to_owned());
    }
    if lower.contains("could not find host")
        || lower.contains("impossibile trovare")
        || lower.contains("nicht finden")
        || lower.contains("introuvable")
        || lower.contains("no pudo encontrar")
    {
        return PingOutcome::Error("Could not resolve host".to_owned());
    }
    if lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("scaduta")
        || lower.contains("zeit")
        || lower.contains("d'attente")
        || lower.contains("agotado")
        || lower.contains("100% packet loss")
    {
        return PingOutcome::Timeout(timeout_ms);
    }
    PingOutcome::Error(
        stdout
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("No recognizable ping result received")
            .trim()
            .to_owned(),
    )
}

fn parse_latency(output: &str) -> Option<u64> {
    // Reply labels are localized on Windows; the delimiter, numeric value and ms are stable.
    for line in output.lines() {
        for (index, delimiter) in line.char_indices() {
            if delimiter != '=' && delimiter != '<' {
                continue;
            }
            let value = line[index + 1..].trim_start();
            let end = value
                .find(|character: char| {
                    !character.is_ascii_digit() && !matches!(character, '.' | ',')
                })
                .unwrap_or(value.len());
            if !value[end..].trim_start().starts_with("ms") {
                continue;
            }
            if let Ok(milliseconds) = value[..end].replace(',', ".").parse::<f64>()
                && milliseconds.is_finite()
                && milliseconds >= 0.0
            {
                return Some(if delimiter == '<' {
                    0
                } else {
                    milliseconds.round() as u64
                });
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

fn settings_save_paths() -> Vec<PathBuf> {
    let mut paths = vec![primary_settings_path()];
    if let Some(fallback) = fallback_settings_path() {
        paths.push(fallback);
    }
    paths
}

fn settings_load_paths() -> Vec<PathBuf> {
    let mut paths = settings_save_paths();
    if let Ok(current_dir) = std::env::current_dir() {
        let legacy = current_dir.join("pingdvd.settings.json");
        if !paths.contains(&legacy) {
            paths.push(legacy);
        }
    }
    paths
}

fn decode_settings(contents: &str) -> Result<Settings, String> {
    serde_json::from_str::<Settings>(contents)
        .map_err(|error| error.to_string())?
        .normalized()
}

fn load_settings_from(paths: &[PathBuf]) -> (Settings, Option<String>) {
    let mut warnings = Vec::new();
    for path in paths {
        let result = match fs::read_to_string(path) {
            Ok(contents) => decode_settings(&contents),
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => Err(error.to_string()),
        };
        match result {
            Ok(settings) => {
                return (
                    settings,
                    (!warnings.is_empty()).then(|| {
                        format!(
                            "{}. Loaded {}; automatic saving disabled until Apply.",
                            warnings.join("; "),
                            path.display()
                        )
                    }),
                );
            }
            Err(error) => warnings.push(format!("{}: {error}", path.display())),
        }
    }
    (
        Settings::default(),
        (!warnings.is_empty()).then(|| {
            format!(
                "Using defaults; automatic saving disabled. {}",
                warnings.join("; ")
            )
        }),
    )
}

fn write_settings_to(settings: &Settings, paths: &[PathBuf]) -> Result<(), String> {
    let settings = settings.normalized()?;
    let contents = serde_json::to_vec_pretty(&settings).map_err(|error| error.to_string())?;
    let mut errors = Vec::new();
    for path in paths {
        match write_settings_file(path, &contents) {
            Ok(()) => return Ok(()),
            Err(error) => errors.push(format!("{}: {error}", path.display())),
        }
    }
    Err(errors.join("; "))
}

fn write_settings_file(path: &Path, contents: &[u8]) -> Result<(), String> {
    match fs::read_to_string(path) {
        Ok(existing) => {
            decode_settings(&existing)
                .map_err(|error| format!("Existing settings preserved: {error}"))?;
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = path.with_extension(format!("json.{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    let result = (|| {
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|error| error.to_string())
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

fn configure_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = SURFACE;
    visuals.window_fill = SURFACE;
    visuals.extreme_bg_color = BACKGROUND;
    visuals.selection.bg_fill = Color32::from_rgb(29, 96, 119);
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.override_text_color = Some(Color32::from_rgb(232, 239, 248));
    ctx.set_visuals(visuals);
    ctx.style_mut(|style| {
        style.spacing.item_spacing = Vec2::new(10.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 6.0);
        style.spacing.interact_size.y = 30.0;
    });
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(800.0, 450.0))
            .with_min_inner_size(Vec2::new(480.0, 400.0))
            .with_icon(app_icon()),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "PingDVD",
        options,
        Box::new(|creation_context| {
            configure_theme(&creation_context.egui_ctx);
            Ok(Box::new(PingApp::new()))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            static NEXT_ID: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "pingdvd-test-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed),
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn file(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn app() -> PingApp {
        PingApp {
            settings: Settings::default(),
            history: VecDeque::new(),
            running: false,
            session_id: 0,
            started_at: None,
            stopped_elapsed: Duration::ZERO,
            worker_settings: None,
            worker_results: None,
            worker_thread: None,
            last_worker_settings: None,
            notice: None,
            autosave: false,
        }
    }

    #[test]
    fn normalizes_hosts_and_rejects_urls_options_and_invalid_numbers() {
        for host in [
            "www.google.it",
            "127.0.0.1",
            "::1",
            "localhost",
            "example.com.",
        ] {
            assert!(valid_host(host), "{host}");
        }
        for host in [
            "",
            "-invalid.example",
            "bad..example",
            "https://example.com",
            "host:80",
        ] {
            assert!(!valid_host(host), "{host}");
        }
        let settings = Settings {
            host: "  localhost  ".to_owned(),
            interval: -1.0,
            timeout: f64::INFINITY,
        };
        assert!(settings.normalized().is_err());
        let settings = Settings {
            timeout: 500.0,
            ..settings
        }
        .normalized()
        .unwrap();
        assert_eq!(settings.host, "localhost");
        assert_eq!(settings.interval, 1.0);
    }

    #[test]
    fn parses_unix_and_localized_windows_replies() {
        for (output, expected) in [
            (
                "64 bytes from ::1: icmp_seq=0 hlim=64 time=0.101 ms",
                Some(0),
            ),
            ("64 bytes from 127.0.0.1: time=12.4 ms", Some(12)),
            ("Reply from 127.0.0.1: bytes=32 time=7ms TTL=128", Some(7)),
            ("Risposta da ::1: durata<1ms", Some(0)),
            ("Antwort von ::1: Zeit=12ms", Some(12)),
            ("Reponse de ::1 : temps=12,6 ms", Some(13)),
            ("Respuesta desde ::1: tiempo=9ms", Some(9)),
            ("Request timed out.", None),
            ("time=NaN ms", None),
            ("time=inf ms", None),
            ("time=-1 ms", None),
        ] {
            assert_eq!(parse_latency(output), expected, "{output}");
        }
    }

    #[test]
    fn distinguishes_reply_timeout_and_errors() {
        assert_eq!(
            classify_ping_output(true, "time=7ms", "", 500),
            PingOutcome::Reply(7)
        );
        for output in [
            "Request timed out.",
            "Richiesta scaduta.",
            "100% packet loss",
        ] {
            assert_eq!(
                classify_ping_output(false, output, "", 500),
                PingOutcome::Timeout(500)
            );
        }
        for (success, stdout, stderr) in [
            (false, "", "ping: unknown host invalid"),
            (false, "", "ping: permission denied"),
            (true, "Destination host unreachable.", ""),
            (false, "Impossibile trovare l'host invalid.", ""),
            (false, "Unrecognized failure", ""),
        ] {
            assert!(matches!(
                classify_ping_output(success, stdout, stderr, 500),
                PingOutcome::Error(_)
            ));
        }
    }

    #[test]
    fn stats_exclude_timeouts_errors_and_expired_samples() {
        let mut app = app();
        assert!(app.history.is_empty());
        app.push_sample(PingOutcome::Reply(999));
        for _ in 0..MAX_HISTORY_SIZE - 2 {
            app.push_sample(PingOutcome::Timeout(500));
        }
        app.push_sample(PingOutcome::Error("DNS error".to_owned()));
        assert!(latency_stats(&VecDeque::from([PingOutcome::Timeout(500)])).is_none());
        app.push_sample(PingOutcome::Reply(10));
        app.push_sample(PingOutcome::Reply(20));
        assert_eq!(app.history.len(), MAX_HISTORY_SIZE);
        assert_eq!(
            latency_stats(&app.history),
            Some(LatencyStats {
                average: 15.0,
                minimum: 10,
                maximum: 20
            })
        );
    }

    #[test]
    fn invalid_live_edits_keep_active_settings_and_host_changes_reject_old_results() {
        let mut app = app();
        app.running = true;
        app.reset_session();
        let old_session = app.session_id;
        app.last_worker_settings = Some(app.settings.clone());
        let (sender, receiver) = watch::channel(WorkerSettings {
            settings: app.settings.clone(),
            session_id: old_session,
        });
        app.worker_settings = Some(sender);
        let (results_tx, results_rx) = mpsc::channel();
        app.worker_results = Some(results_rx);
        app.settings.host = "https://invalid".to_owned();
        app.update_worker_settings();
        assert!(!receiver.has_changed().unwrap());
        assert_eq!(
            app.last_worker_settings.as_ref().unwrap().host,
            DEFAULT_HOST
        );

        app.settings.host = "localhost".to_owned();
        app.push_sample(PingOutcome::Reply(10));
        app.update_worker_settings();
        assert!(app.history.is_empty());
        results_tx
            .send(PingResult {
                session_id: old_session,
                outcome: PingOutcome::Reply(999),
            })
            .unwrap();
        results_tx
            .send(PingResult {
                session_id: app.session_id,
                outcome: PingOutcome::Reply(5),
            })
            .unwrap();
        app.poll_worker();
        assert_eq!(app.history, VecDeque::from([PingOutcome::Reply(5)]));

        let session_id = app.session_id;
        app.settings.timeout = 750.0;
        app.update_worker_settings();
        assert_eq!(app.session_id, session_id);
        assert_eq!(receiver.borrow().settings.timeout, 750.0);
    }

    #[test]
    fn invalid_start_does_not_replace_the_host_or_create_a_worker() {
        let mut app = app();
        app.settings.host = "--invalid".to_owned();
        app.start();
        assert!(!app.running);
        assert!(app.worker_thread.is_none());
        assert_eq!(app.settings.host, "--invalid");
    }

    #[test]
    fn stop_freezes_duration_and_disconnected_worker_stops_the_session() {
        let mut app = app();
        app.running = true;
        app.started_at = Some(Instant::now() - Duration::from_secs(4));
        let (sender, receiver) = mpsc::channel();
        app.worker_results = Some(receiver);
        drop(sender);
        app.poll_worker();
        assert!(!app.running);
        assert!(app.started_at.is_none());
        let elapsed = app.elapsed_duration();
        assert_eq!(app.elapsed_duration(), elapsed);
        assert!(elapsed >= Duration::from_secs(4));
        assert!(app.notice.as_ref().unwrap().text.contains("disconnected"));
    }

    #[test]
    fn legacy_settings_remain_readable_and_values_are_clamped() {
        let settings =
            decode_settings(r#"{"Host":"example.com","Interval":0,"TimeOut":250000}"#).unwrap();
        assert_eq!(
            settings,
            Settings {
                host: "example.com".to_owned(),
                interval: 1.0,
                timeout: MAX_SETTING_MS
            }
        );
    }

    #[test]
    fn recovers_fallback_without_overwriting_corrupt_settings() {
        let dir = TestDirectory::new();
        let primary = dir.file("primary.json");
        let fallback = dir.file("fallback.json");
        fs::write(&primary, "{broken").unwrap();
        write_settings_file(&fallback, br#"{"host":"localhost"}"#).unwrap();
        let paths = [primary.clone(), fallback.clone()];
        let (mut settings, warning) = load_settings_from(&paths);
        assert_eq!(settings.host, "localhost");
        assert!(warning.unwrap().contains("primary.json"));
        settings.interval = 123.0;
        write_settings_to(&settings, &paths).unwrap();
        assert_eq!(fs::read_to_string(&primary).unwrap(), "{broken");
        assert_eq!(load_settings_from(&[fallback]).0.interval, 123.0);
    }

    #[test]
    fn write_failure_preserves_existing_file_and_does_not_delete_other_temporary_files() {
        let dir = TestDirectory::new();
        let path = dir.file("settings.json");
        write_settings_file(&path, br#"{"host":"localhost"}"#).unwrap();
        let temporary = path.with_extension(format!("json.{}.tmp", std::process::id()));
        fs::write(&temporary, "another writer").unwrap();
        assert!(write_settings_file(&path, br#"{"host":"127.0.0.1"}"#).is_err());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            r#"{"host":"localhost"}"#
        );
        assert_eq!(fs::read_to_string(&temporary).unwrap(), "another writer");
        fs::remove_file(temporary).unwrap();
        write_settings_file(&path, br#"{"host":"127.0.0.1"}"#).unwrap();
        assert_eq!(load_settings_from(&[path]).0.host, "127.0.0.1");
    }

    #[test]
    fn unreadable_path_and_invalid_host_fall_back_with_a_warning() {
        let dir = TestDirectory::new();
        let invalid = dir.file("invalid.json");
        fs::write(&invalid, r#"{"host":"--invalid"}"#).unwrap();
        let (settings, warning) = load_settings_from(&[dir.0.clone(), invalid]);
        assert_eq!(settings, Settings::default());
        assert!(warning.unwrap().contains("automatic saving disabled"));
    }

    #[test]
    fn closed_settings_channel_cancels_worker_without_a_network_dependency() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (sender, receiver) = watch::channel(WorkerSettings {
            settings: Settings::default(),
            session_id: 0,
        });
        let (results, _) = mpsc::channel();
        drop(sender);
        runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(1), run_ping_loop(receiver, results))
                .await
                .expect("a closed channel must cancel the worker");
        });
    }

    #[test]
    fn layout_handles_empty_failed_and_single_reply_history_at_small_and_hidpi_sizes() {
        let ctx = egui::Context::default();
        configure_theme(&ctx);
        let mut app = app();
        for size in [
            Vec2::new(480.0, 400.0),
            Vec2::new(700.0, 450.0),
            Vec2::new(800.0, 450.0),
        ] {
            for pixels_per_point in [1.0, 2.0] {
                ctx.set_pixels_per_point(pixels_per_point);
                for history in [
                    VecDeque::new(),
                    VecDeque::from([PingOutcome::Timeout(500)]),
                    VecDeque::from([PingOutcome::Reply(0)]),
                ] {
                    app.history = history;
                    let _ = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, size)),
                            ..Default::default()
                        },
                        |ctx| app.show(ctx),
                    );
                    let output = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, size)),
                            ..Default::default()
                        },
                        |ctx| app.show(ctx),
                    );
                    let texts: Vec<_> = output
                        .shapes
                        .iter()
                        .filter_map(|shape| match &shape.shape {
                            egui::Shape::Text(text) => Some(text.galley.text()),
                            _ => None,
                        })
                        .collect();
                    assert!(texts.contains(&"Stopped"));
                    for label in ["Host", "Timeout", "Interval", "Start", "Apply", "Reset"] {
                        assert!(texts.contains(&label), "{size:?}: missing {label}");
                        for shape in &output.shapes {
                            if let egui::Shape::Text(text) = &shape.shape
                                && text.galley.text() == label
                            {
                                let bounds = shape.shape.visual_bounding_rect();
                                assert!(bounds.min.x >= 0.0 && bounds.max.x <= size.x);
                                assert!(bounds.min.y >= 0.0 && bounds.max.y <= size.y);
                            }
                        }
                    }
                    assert!(
                        texts.iter().any(|text| text.starts_with("SESSION")),
                        "{size:?} at {pixels_per_point}x: {texts:?}"
                    );
                    assert!(
                        output
                            .shapes
                            .iter()
                            .all(|shape| shape.clip_rect.is_finite())
                    );
                    assert!(!texts.iter().any(|text| text.contains("NaN")));
                }
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn timeout_and_cancellation_kill_an_inflight_child_process() {
        let dir = TestDirectory::new();
        for cancel in [false, true] {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let pid = runtime.block_on(async {
                let pid_file = dir.file(if cancel { "cancel.pid" } else { "timeout.pid" });
                let mut command = Command::new("/bin/sh");
                command
                    .args(["-c", "echo $$ > \"$1\"; exec sleep 30", "pingdvd-test"])
                    .arg(&pid_file)
                    .kill_on_drop(true);
                let (cancel_tx, mut cancel_rx) = watch::channel(WorkerSettings {
                    settings: Settings::default(),
                    session_id: 0,
                });
                let pending = tokio::spawn(async move {
                    collect_ping_output(
                        command,
                        if cancel { 30_000 } else { 300 },
                        Some(&mut cancel_rx),
                    )
                    .await
                });
                tokio::time::timeout(Duration::from_secs(2), async {
                    while !pid_file.exists() {
                        sleep(Duration::from_millis(5)).await;
                    }
                })
                .await
                .unwrap();
                if cancel {
                    drop(cancel_tx);
                    assert_eq!(pending.await.unwrap(), None);
                } else {
                    assert_eq!(pending.await.unwrap(), Some(PingOutcome::Timeout(300)));
                }
                fs::read_to_string(&pid_file).unwrap()
            });
            drop(runtime);
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                let alive = std::process::Command::new("/bin/kill")
                    .args(["-0", pid.trim()])
                    .output()
                    .unwrap();
                if !alive.status.success() {
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "child {pid} survived runtime shutdown"
                );
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    #[test]
    #[ignore = "manual smoke test: requires the system ping commands and loopback networking"]
    fn loopback_ipv4_and_ipv6_reply_and_worker_can_restart() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        for host in ["127.0.0.1", "::1"] {
            let outcome = runtime.block_on(ping_once(host, 2000));
            assert!(
                matches!(outcome, PingOutcome::Reply(_)),
                "{host}: {outcome:?}"
            );
        }
        let mut app = app();
        app.settings.host = "127.0.0.1".to_owned();
        for _ in 0..2 {
            app.start();
            let result = app
                .worker_results
                .as_ref()
                .unwrap()
                .recv_timeout(Duration::from_secs(3))
                .unwrap();
            assert!(matches!(result.outcome, PingOutcome::Reply(_)));
            app.stop();
            assert!(!app.running);
            assert!(app.worker_thread.is_none());
        }
    }
}
