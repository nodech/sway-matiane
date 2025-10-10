use chrono::Local;
use log::{LevelFilter, Log, Metadata, Record};

pub struct Logger {
    level: LevelFilter,
    stderr: bool,
    stdout: bool,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level().to_level_filter() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp = Local::now();
        let level = record.level();
        let target = if !record.target().is_empty() {
            record.target()
        } else {
            record.module_path().unwrap_or_default()
        };

        let formatted =
            format!("{} [{}] ({}) {}", timestamp, level, target, record.args());

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
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        LoggerBuilder {
            level: LevelFilter::Off,
            stderr: false,
            stdout: false,
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

    pub fn build(self) -> Logger {
        Logger {
            level: self.level,
            stderr: self.stderr,
            stdout: self.stdout,
        }
    }
}
