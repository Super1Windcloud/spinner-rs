use std::io::{self, IsTerminal, Write};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossterm::{
    cursor::{Hide, Show},
    queue,
    style::{Color as TermColor, Print, ResetColor, SetForegroundColor},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stream {
    Stdout,
    Stderr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Color {
    fn into_term(self) -> TermColor {
        match self {
            Self::Black => TermColor::Black,
            Self::Red => TermColor::Red,
            Self::Green => TermColor::Green,
            Self::Yellow => TermColor::Yellow,
            Self::Blue => TermColor::Blue,
            Self::Magenta => TermColor::Magenta,
            Self::Cyan => TermColor::Cyan,
            Self::White => TermColor::White,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpinnerStyle {
    pub interval: Duration,
    pub frames: Vec<String>,
}

impl SpinnerStyle {
    pub fn new<I, S>(interval: Duration, frames: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            interval,
            frames: frames.into_iter().map(Into::into).collect(),
        }
    }

    pub fn dots() -> Self {
        Self::new(
            Duration::from_millis(80),
            ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
        )
    }

    pub fn line() -> Self {
        Self::new(Duration::from_millis(130), ["-", "\\", "|", "/"])
    }

    pub fn arrow() -> Self {
        Self::new(
            Duration::from_millis(100),
            ["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
        )
    }
}

impl Default for SpinnerStyle {
    fn default() -> Self {
        Self::dots()
    }
}

#[derive(Clone, Debug)]
pub struct SpinnerOptions {
    pub text: String,
    pub prefix_text: String,
    pub suffix_text: String,
    pub color: Option<Color>,
    pub spinner: SpinnerStyle,
    pub indent: usize,
    pub hide_cursor: bool,
    pub enabled: bool,
    pub stream: Stream,
}

impl SpinnerOptions {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            prefix_text: String::new(),
            suffix_text: String::new(),
            color: Some(Color::Cyan),
            spinner: SpinnerStyle::default(),
            indent: 0,
            hide_cursor: true,
            enabled: true,
            stream: Stream::Stderr,
        }
    }

    pub fn builder() -> SpinnerBuilder {
        SpinnerBuilder::default()
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn with_prefix_text(mut self, text: impl Into<String>) -> Self {
        self.prefix_text = text.into();
        self
    }

    pub fn with_suffix_text(mut self, text: impl Into<String>) -> Self {
        self.suffix_text = text.into();
        self
    }

    pub fn with_color(mut self, color: Option<Color>) -> Self {
        self.color = color;
        self
    }

    pub fn with_spinner(mut self, spinner: SpinnerStyle) -> Self {
        self.spinner = spinner;
        self
    }

    pub fn with_indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    pub fn with_hide_cursor(mut self, hide_cursor: bool) -> Self {
        self.hide_cursor = hide_cursor;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_stream(mut self, stream: Stream) -> Self {
        self.stream = stream;
        self
    }
}

impl Default for SpinnerOptions {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Clone, Debug, Default)]
pub struct SpinnerBuilder {
    options: SpinnerOptions,
}

impl SpinnerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.options.text = text.into();
        self
    }

    pub fn prefix_text(mut self, text: impl Into<String>) -> Self {
        self.options.prefix_text = text.into();
        self
    }

    pub fn suffix_text(mut self, text: impl Into<String>) -> Self {
        self.options.suffix_text = text.into();
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.options.color = Some(color);
        self
    }

    pub fn no_color(mut self) -> Self {
        self.options.color = None;
        self
    }

    pub fn spinner(mut self, spinner: SpinnerStyle) -> Self {
        self.options.spinner = spinner;
        self
    }

    pub fn indent(mut self, indent: usize) -> Self {
        self.options.indent = indent;
        self
    }

    pub fn hide_cursor(mut self, hide_cursor: bool) -> Self {
        self.options.hide_cursor = hide_cursor;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.options.enabled = enabled;
        self
    }

    pub fn stream(mut self, stream: Stream) -> Self {
        self.options.stream = stream;
        self
    }

    pub fn build(self) -> Spinner {
        Spinner::with_options(self.options)
    }

    pub fn build_options(self) -> SpinnerOptions {
        self.options
    }
}

#[derive(Clone, Debug)]
struct SpinnerState {
    text: String,
    prefix_text: String,
    suffix_text: String,
    frame_index: usize,
    color: Option<Color>,
    spinner: SpinnerStyle,
    indent: usize,
    hide_cursor: bool,
    enabled: bool,
    stream: Stream,
    last_width: usize,
}

impl From<SpinnerOptions> for SpinnerState {
    fn from(value: SpinnerOptions) -> Self {
        Self {
            text: value.text,
            prefix_text: value.prefix_text,
            suffix_text: value.suffix_text,
            frame_index: 0,
            color: value.color,
            spinner: value.spinner,
            indent: value.indent,
            hide_cursor: value.hide_cursor,
            enabled: value.enabled,
            stream: value.stream,
            last_width: 0,
        }
    }
}

pub struct Spinner {
    state: Arc<Mutex<SpinnerState>>,
    running: Arc<AtomicBool>,
    worker: Option<JoinHandle<io::Result<()>>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpinnerSnapshot {
    pub text: String,
    pub prefix_text: String,
    pub suffix_text: String,
    pub color: Option<Color>,
    pub spinner: SpinnerStyle,
    pub indent: usize,
    pub hide_cursor: bool,
    pub enabled: bool,
    pub stream: Stream,
    pub is_spinning: bool,
}

impl Spinner {
    pub fn new(text: impl Into<String>) -> Self {
        Self::with_options(SpinnerOptions::new(text))
    }

    pub fn builder() -> SpinnerBuilder {
        SpinnerBuilder::default()
    }

    pub fn with_options(options: SpinnerOptions) -> Self {
        Self {
            state: Arc::new(Mutex::new(options.into())),
            running: Arc::new(AtomicBool::new(false)),
            worker: None,
        }
    }

    pub fn start(&mut self) -> io::Result<&mut Self> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(self);
        }

        let state = Arc::clone(&self.state);
        let running = Arc::clone(&self.running);

        self.worker = Some(thread::spawn(move || {
            let mut cursor_hidden = false;

            while running.load(Ordering::SeqCst) {
                let mut guard = state.lock().expect("spinner state poisoned");
                let enabled = guard.enabled && is_terminal(guard.stream);
                let interval = guard.spinner.interval;

                if enabled && guard.hide_cursor && !cursor_hidden {
                    let writer = writer_for(guard.stream);
                    let mut writer = writer.lock().expect("writer poisoned");
                    queue!(&mut *writer, Hide)?;
                    writer.flush()?;
                    cursor_hidden = true;
                }

                render_active(&mut guard)?;
                drop(guard);
                thread::sleep(interval);
            }

            let guard = state.lock().expect("spinner state poisoned");
            if cursor_hidden && guard.enabled && is_terminal(guard.stream) {
                let writer = writer_for(guard.stream);
                let mut writer = writer.lock().expect("writer poisoned");
                queue!(&mut *writer, Show)?;
                writer.flush()?;
            }

            Ok(())
        }));

        Ok(self)
    }

    pub fn stop(&mut self) -> io::Result<&mut Self> {
        self.stop_internal()?;
        self.clear()?;
        Ok(self)
    }

    pub fn clear(&mut self) -> io::Result<&mut Self> {
        {
            let mut guard = self.state.lock().expect("spinner state poisoned");
            clear_line(&mut guard)?;
        }
        Ok(self)
    }

    pub fn stop_and_persist<T>(
        &mut self,
        symbol: impl Into<String>,
        text: Option<T>,
    ) -> io::Result<&mut Self>
    where
        T: Into<String>,
    {
        self.stop_internal()?;
        {
            let mut guard = self.state.lock().expect("spinner state poisoned");
            if let Some(text) = text {
                guard.text = text.into();
            }
            render_persisted(&mut guard, &symbol.into(), None)?;
        }
        Ok(self)
    }

    pub fn succeed<T>(&mut self, text: Option<T>) -> io::Result<&mut Self>
    where
        T: Into<String>,
    {
        self.stop_internal()?;
        {
            let mut guard = self.state.lock().expect("spinner state poisoned");
            if let Some(text) = text {
                guard.text = text.into();
            }
            render_persisted(&mut guard, "✔", Some(Color::Green))?;
        }
        Ok(self)
    }

    pub fn fail<T>(&mut self, text: Option<T>) -> io::Result<&mut Self>
    where
        T: Into<String>,
    {
        self.stop_internal()?;
        {
            let mut guard = self.state.lock().expect("spinner state poisoned");
            if let Some(text) = text {
                guard.text = text.into();
            }
            render_persisted(&mut guard, "✖", Some(Color::Red))?;
        }
        Ok(self)
    }

    pub fn warn<T>(&mut self, text: Option<T>) -> io::Result<&mut Self>
    where
        T: Into<String>,
    {
        self.stop_internal()?;
        {
            let mut guard = self.state.lock().expect("spinner state poisoned");
            if let Some(text) = text {
                guard.text = text.into();
            }
            render_persisted(&mut guard, "⚠", Some(Color::Yellow))?;
        }
        Ok(self)
    }

    pub fn info<T>(&mut self, text: Option<T>) -> io::Result<&mut Self>
    where
        T: Into<String>,
    {
        self.stop_internal()?;
        {
            let mut guard = self.state.lock().expect("spinner state poisoned");
            if let Some(text) = text {
                guard.text = text.into();
            }
            render_persisted(&mut guard, "ℹ", Some(Color::Blue))?;
        }
        Ok(self)
    }

    pub fn text(&self) -> String {
        self.state
            .lock()
            .expect("spinner state poisoned")
            .text
            .clone()
    }

    pub fn set_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.state.lock().expect("spinner state poisoned").text = text.into();
        self
    }

    pub fn set_prefix_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.state
            .lock()
            .expect("spinner state poisoned")
            .prefix_text = text.into();
        self
    }

    pub fn set_suffix_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.state
            .lock()
            .expect("spinner state poisoned")
            .suffix_text = text.into();
        self
    }

    pub fn set_color(&mut self, color: Option<Color>) -> &mut Self {
        self.state.lock().expect("spinner state poisoned").color = color;
        self
    }

    pub fn set_spinner(&mut self, spinner: SpinnerStyle) -> &mut Self {
        {
            let mut guard = self.state.lock().expect("spinner state poisoned");
            guard.spinner = spinner;
            guard.frame_index = 0;
        }
        self
    }

    pub fn snapshot(&self) -> SpinnerSnapshot {
        let guard = self.state.lock().expect("spinner state poisoned");

        SpinnerSnapshot {
            text: guard.text.clone(),
            prefix_text: guard.prefix_text.clone(),
            suffix_text: guard.suffix_text.clone(),
            color: guard.color,
            spinner: guard.spinner.clone(),
            indent: guard.indent,
            hide_cursor: guard.hide_cursor,
            enabled: guard.enabled,
            stream: guard.stream,
            is_spinning: self.is_spinning(),
        }
    }

    pub fn is_spinning(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn stop_internal(&mut self) -> io::Result<()> {
        self.running.store(false, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            match worker.join() {
                Ok(result) => result?,
                Err(_) => return Err(io::Error::other("spinner thread panicked")),
            }
        }
        Ok(())
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn render_active(state: &mut SpinnerState) -> io::Result<()> {
    if !state.enabled || !is_terminal(state.stream) {
        return Ok(());
    }

    let frame = if state.spinner.frames.is_empty() {
        "-".to_string()
    } else {
        state.spinner.frames[state.frame_index % state.spinner.frames.len()].clone()
    };
    state.frame_index = state.frame_index.wrapping_add(1);

    let line = compose_line(state, &frame);
    write_line(state, &line, state.color)?;
    Ok(())
}

fn render_persisted(
    state: &mut SpinnerState,
    symbol: &str,
    symbol_color: Option<Color>,
) -> io::Result<()> {
    let line = compose_line(state, symbol);
    write_line(state, &line, symbol_color)?;
    let writer = writer_for(state.stream);
    let mut writer = writer.lock().expect("writer poisoned");
    writer.write_all(b"\n")?;
    writer.flush()?;
    state.last_width = 0;
    Ok(())
}

fn clear_line(state: &mut SpinnerState) -> io::Result<()> {
    if !state.enabled || !is_terminal(state.stream) {
        return Ok(());
    }

    let writer = writer_for(state.stream);
    let mut writer = writer.lock().expect("writer poisoned");
    let clear_width = state.last_width.max(1);
    write!(&mut *writer, "\r{}\r", " ".repeat(clear_width))?;
    writer.flush()?;
    state.last_width = 0;
    Ok(())
}

fn write_line(state: &mut SpinnerState, line: &str, symbol_color: Option<Color>) -> io::Result<()> {
    let writer = writer_for(state.stream);
    let mut writer = writer.lock().expect("writer poisoned");
    let clear_width = state.last_width.max(line.chars().count());

    write!(&mut *writer, "\r{}\r", " ".repeat(clear_width))?;
    if let Some(color) = symbol_color {
        queue!(&mut *writer, SetForegroundColor(color.into_term()))?;
    }
    queue!(&mut *writer, Print(line))?;
    if symbol_color.is_some() {
        queue!(&mut *writer, ResetColor)?;
    }
    writer.flush()?;
    state.last_width = line.chars().count();
    Ok(())
}

fn compose_line(state: &SpinnerState, symbol: &str) -> String {
    let indent = " ".repeat(state.indent);
    let mut line = format!("{indent}{symbol}");

    if !state.prefix_text.is_empty() {
        line.push(' ');
        line.push_str(&state.prefix_text);
    }

    if !state.text.is_empty() {
        line.push(' ');
        line.push_str(&state.text);
    }

    if !state.suffix_text.is_empty() {
        line.push(' ');
        line.push_str(&state.suffix_text);
    }

    line
}

fn is_terminal(stream: Stream) -> bool {
    match stream {
        Stream::Stdout => io::stdout().is_terminal(),
        Stream::Stderr => io::stderr().is_terminal(),
    }
}

fn writer_for(stream: Stream) -> &'static Mutex<Box<dyn Write + Send>> {
    use std::sync::OnceLock;

    static STDOUT: OnceLock<Mutex<Box<dyn Write + Send>>> = OnceLock::new();
    static STDERR: OnceLock<Mutex<Box<dyn Write + Send>>> = OnceLock::new();

    match stream {
        Stream::Stdout => STDOUT.get_or_init(|| Mutex::new(Box::new(io::stdout()))),
        Stream::Stderr => STDERR.get_or_init(|| Mutex::new(Box::new(io::stderr()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_new_sets_expected_defaults() {
        let options = SpinnerOptions::new("loading");

        assert_eq!(options.text, "loading");
        assert_eq!(options.prefix_text, "");
        assert_eq!(options.suffix_text, "");
        assert_eq!(options.color, Some(Color::Cyan));
        assert_eq!(options.spinner.frames.len(), 10);
        assert_eq!(options.indent, 0);
        assert!(options.hide_cursor);
        assert!(options.enabled);
        assert_eq!(options.stream, Stream::Stderr);
    }

    #[test]
    fn style_presets_are_available() {
        let dots = SpinnerStyle::dots();
        let line = SpinnerStyle::line();
        let arrow = SpinnerStyle::arrow();

        assert_eq!(dots.frames.len(), 10);
        assert_eq!(line.frames, vec!["-", "\\", "|", "/"]);
        assert_eq!(arrow.frames.len(), 8);
    }

    #[test]
    fn custom_style_constructor_collects_frames() {
        let style = SpinnerStyle::new(Duration::from_millis(50), ["a", "b", "c"]);

        assert_eq!(style.interval, Duration::from_millis(50));
        assert_eq!(style.frames, vec!["a", "b", "c"]);
    }

    #[test]
    fn builder_build_options_sets_all_fields() {
        let options = SpinnerBuilder::new()
            .text("compile")
            .prefix_text("worker")
            .suffix_text("queued")
            .color(Color::Green)
            .spinner(SpinnerStyle::arrow())
            .indent(4)
            .hide_cursor(false)
            .enabled(false)
            .stream(Stream::Stdout)
            .build_options();

        assert_eq!(options.text, "compile");
        assert_eq!(options.prefix_text, "worker");
        assert_eq!(options.suffix_text, "queued");
        assert_eq!(options.color, Some(Color::Green));
        assert_eq!(options.spinner, SpinnerStyle::arrow());
        assert_eq!(options.indent, 4);
        assert!(!options.hide_cursor);
        assert!(!options.enabled);
        assert_eq!(options.stream, Stream::Stdout);
    }

    #[test]
    fn compose_line_combines_all_segments() {
        let state = SpinnerState {
            text: "compile".into(),
            prefix_text: "job-7".into(),
            suffix_text: "done".into(),
            frame_index: 0,
            color: Some(Color::Blue),
            spinner: SpinnerStyle::line(),
            indent: 2,
            hide_cursor: false,
            enabled: true,
            stream: Stream::Stderr,
            last_width: 0,
        };

        assert_eq!(compose_line(&state, "*"), "  * job-7 compile done");
    }

    #[test]
    fn spinner_mutators_update_text_fields() {
        let mut spinner = Spinner::new("boot");

        spinner
            .set_text("compile")
            .set_prefix_text("worker")
            .set_suffix_text("queued")
            .set_color(Some(Color::Green))
            .set_spinner(SpinnerStyle::arrow());

        let guard = spinner.state.lock().expect("spinner state poisoned");
        assert_eq!(guard.text, "compile");
        assert_eq!(guard.prefix_text, "worker");
        assert_eq!(guard.suffix_text, "queued");
        assert_eq!(guard.color, Some(Color::Green));
        assert_eq!(guard.spinner.frames.len(), 8);
    }

    #[test]
    fn start_and_stop_toggle_running_state() {
        let mut spinner = Spinner::with_options(SpinnerOptions {
            text: "build".into(),
            enabled: false,
            ..SpinnerOptions::default()
        });

        spinner.start().expect("spinner should start");
        assert!(spinner.is_spinning());

        spinner.stop().expect("spinner should stop");
        assert!(!spinner.is_spinning());
    }

    #[test]
    fn snapshot_reflects_mutations() {
        let mut spinner = Spinner::builder()
            .text("start")
            .prefix_text("unit")
            .suffix_text("pending")
            .build();

        spinner.set_text("finish").set_color(Some(Color::Magenta));

        let snapshot = spinner.snapshot();
        assert_eq!(snapshot.text, "finish");
        assert_eq!(snapshot.prefix_text, "unit");
        assert_eq!(snapshot.suffix_text, "pending");
        assert_eq!(snapshot.color, Some(Color::Magenta));
        assert!(!snapshot.is_spinning);
    }
}
