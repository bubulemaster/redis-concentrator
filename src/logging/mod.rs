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
        "syslog" =>
            match &config.log.syslog_id {
                Some(id) =>
                    match &config.log.syslog_who {
                        Some(who) => Some(create_syslog_log(who.to_string(), id.to_string())),
                        None => None
                    }
                None => None
            },
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
/*
    // slog_stdlog uses the logger from slog_scope, so set a logger there
    let _guard = slog_scope::set_global_logger(logger);

    // register slog_stdlog as the log handler with the log crate
    slog_stdlog::init().unwrap();*/
}

fn create_console_log() -> slog::Logger {
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    slog::Logger::root(drain, o!())

    /*
    // slog_stdlog uses the logger from slog_scope, so set a logger there
    let _guard = slog_scope::set_global_logger(logger);

    // register slog_stdlog as the log handler with the log crate
    slog_stdlog::init().unwrap();*/
}

fn create_syslog_log(who: String, id: String) -> slog::Logger {
    // TODO allow change facility
    let syslog = slog_syslog::unix_3164(Facility::LOG_USER).unwrap();
    let root = slog::Logger::root(syslog.fuse(), o!());
    root.new(o!("who" => "slog-syslog test", "build-id" => id))

    /*
    // slog_stdlog uses the logger from slog_scope, so set a logger there
    let _guard = slog_scope::set_global_logger(logger);

    // register slog_stdlog as the log handler with the log crate
    slog_stdlog::init().unwrap();*/
}