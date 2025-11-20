use chrono::Local;
use log::{LevelFilter, Log, Metadata, Record};

pub struct Logger {
    level: LevelFilter,
    stderr: bool,
    stdout: bool,
    thread: bool,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level().to_level_filter() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp =
            Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let level = format!("[{}]", record.level());
        let target = if !record.target().is_empty() {
            format!("comp={} ", record.target())
        } else {
            format!("comp={} ", record.module_path().unwrap_or_default())
        };

        let thread = if self.thread {
            let thread = std::thread::current();
            let name = thread.name().unwrap_or("?");
            format!("t={} ", name)
        } else {
            "".to_string()
        };

        let formatted = format!(
            "{} {:<7} {}{} {}",
            timestamp,
            level,
            thread,
            target,
            record.args()
        );

        if self.stderr {
            eprintln!("{}", formatted);
        }

        if self.stdout {
            println!("{}", formatted);
        }
    }

    fn flush(&self) {}
}

pub struct LoggerBuilder {
    level: LevelFilter,
    stderr: bool,
    stdout: bool,
    thread: bool,
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        LoggerBuilder {
            level: LevelFilter::Off,
            stderr: false,
            stdout: false,
            thread: false,
        }
    }
}

impl LoggerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    pub fn to_stderr(mut self, enabled: bool) -> Self {
        self.stderr = enabled;
        self
    }

    pub fn to_stdout(mut self, enabled: bool) -> Self {
        self.stdout = enabled;
        self
    }

    pub fn with_threads(mut self, enabled: bool) -> Self {
        self.thread = enabled;
        self
    }

    pub fn build(self) -> Logger {
        Logger {
            level: self.level,
            stderr: self.stderr,
            stdout: self.stdout,
            thread: self.thread,
        }
    }
}

pub fn init_global_logger(
    level: LevelFilter,
) -> Result<(), log::SetLoggerError> {
    let logger = LoggerBuilder::new()
        .with_level(level)
        .to_stderr(true)
        .with_threads(true)
        .build();

    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level);

    Ok(())
}
