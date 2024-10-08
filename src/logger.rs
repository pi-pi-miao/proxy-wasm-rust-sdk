// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::hostcalls;
use crate::types::LogLevel;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};

struct Logger;

static LOGGER: Logger = Logger;
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub(crate) fn set_log_level(level: LogLevel) -> Result<(), Box<dyn std::error::Error>> {
    if !INITIALIZED.load(Ordering::Relaxed) {
        log::set_logger(&LOGGER)
            .map_err(|e| format!("Failed to set logger: {}", e))?;

        panic::set_hook(Box::new(|panic_info| {
            if let Err(e) = hostcalls::log(LogLevel::Critical, &panic_info.to_string()) {
                eprintln!("Failed to log panic info: {:?}", e);
            }
        }));

        INITIALIZED.store(true, Ordering::Relaxed);
    }
    LOGGER.set_log_level(level);
    Ok(())
}


impl Logger {
    pub fn set_log_level(&self, level: LogLevel) {
        let filter = match level {
            LogLevel::Trace => log::LevelFilter::Trace,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Critical => log::LevelFilter::Off,
        };
        log::set_max_level(filter);
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let level = match record.level() {
            log::Level::Trace => LogLevel::Trace,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Info => LogLevel::Info,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Error => LogLevel::Error,
        };
        let message = record.args().to_string();
        if let Err(e) = hostcalls::log(level, &message) {
            // 如果日志记录失败，我们可以尝试打印到标准错误
            eprintln!("Failed to log message: {}. Error: {:?}", message, e);
        }
    }


    fn flush(&self) {}
}
