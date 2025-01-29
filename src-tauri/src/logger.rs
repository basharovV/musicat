use log::{max_level, Level, LevelFilter, Record};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, Deserialize_repr, Serialize_repr, PartialEq)]
#[repr(u16)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[tauri::command]
pub fn max_log_level() -> LogLevel {
    match max_level() {
        LevelFilter::Off => LogLevel::Off,
        LevelFilter::Error => LogLevel::Error,
        LevelFilter::Warn => LogLevel::Warn,
        LevelFilter::Info => LogLevel::Info,
        LevelFilter::Debug => LogLevel::Debug,
        LevelFilter::Trace => LogLevel::Trace,
    }
}

#[tauri::command]
pub fn write_log(
    level: LogLevel,
    message: String,
    file: Option<&str>,
    line: Option<u32>,
    caller: Option<&str>,
) {
    let level_f = match level {
        LogLevel::Off => {
            return;
        }
        LogLevel::Error => Level::Error,
        LogLevel::Warn => Level::Warn,
        LogLevel::Info => Level::Info,
        LogLevel::Debug => Level::Debug,
        LogLevel::Trace => Level::Trace,
    };
    let caller = caller.unwrap_or("webview");

    (|args: std::fmt::Arguments| {
        let record = Record::builder()
            .target("app::frontend")
            .level(level_f)
            .args(args)
            .file(file)
            .line(line)
            .module_path(Some(caller))
            .build();

        log::logger().log(&record);
    })(format_args!("{}", message));
}
