//! This module contain log's routine.
//!
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_syslog;
extern crate slog_term;

use slog::{Drain, Fuse, Level};
use slog_syslog::Facility;
use std::fs::OpenOptions;

use crate::config::Config;

pub fn create_log(config: &Config) -> Option<slog::Logger> {
    // Get log level
    let log_level = match config.log.level.as_str().to_lowercase().as_str() {
        "critical" => Level::Critical,
        "error" => Level::Error,
        "warning" => Level::Warning,
        "info" => Level::Info,
        "debug" => Level::Debug,
        "trace" => Level::Trace,
        e => {
            println!("Debug level '{}' not supported!", e);
            return None;
        }
    };

    match config.log.log_type.as_str() {
        "console" => Some(create_console_log(log_level)),
        "file" => match &config.log.file {
            Some(s) => Some(create_file_log(s.to_string(), log_level)),
            None => None,
        },
        "syslog" => Some(create_syslog_log(log_level)),
        e => {
            println!("Log type '{}' not supported!", e);
            return None;
        }
    }
}

fn create_file_log(filename: String, log_level: Level) -> slog::Logger {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)
        .unwrap();

    // create logger
    let decorator = slog_term::PlainSyncDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog::LevelFilter::new(drain, log_level).map(Fuse);

    slog::Logger::root(drain, o!())
}

fn create_console_log(log_level: Level) -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog::LevelFilter::new(drain, log_level).map(Fuse);
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, o!())
}

fn create_syslog_log(log_level: Level) -> slog::Logger {
    // TODO allow change facility
    let syslog = slog_syslog::unix_3164(Facility::LOG_USER).unwrap();
    let drain = slog::LevelFilter::new(syslog.fuse(), log_level).map(Fuse);
    let drain = slog::Logger::root(drain, o!());
    //root.new(o!())

    slog::Logger::root(drain, o!())
}
