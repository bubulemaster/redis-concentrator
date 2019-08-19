//! This module contain log's routine.
//!
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_term;
extern crate slog_syslog;

use std::fs::OpenOptions;
use slog::Drain;
use slog_syslog::Facility;

use crate::config::{Config, ConfigLog};

pub fn create_log(config: &Config) -> Option<slog::Logger> {
    match config.log.log_type.as_str() {
        "console" => Some(create_console_log()),
        "file" =>
            match &config.log.file {
                Some(s) => Some(create_file_log(s.to_string())),
                None => None
            },
        "syslog" => Some(create_syslog_log()),
        e => // TODO
        None
    }
}

fn create_file_log(filename: String) -> slog::Logger  {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)
        .unwrap();

    // create logger
    let decorator = slog_term::PlainSyncDecorator::new(file);
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    slog::Logger::root(drain, o!())
}

fn create_console_log() -> slog::Logger {
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, o!())
}

fn create_syslog_log() -> slog::Logger {
    // TODO allow change facility
    let syslog = slog_syslog::unix_3164(Facility::LOG_USER).unwrap();
    let root = slog::Logger::root(syslog.fuse(), o!());
    root.new(o!())
}