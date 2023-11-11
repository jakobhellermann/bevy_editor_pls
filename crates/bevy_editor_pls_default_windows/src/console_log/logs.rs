use std::str::FromStr;
use std::sync::{Arc, OnceLock, RwLock};

use bevy::prelude::*;
use bevy::utils::HashMap;

static FILTER: OnceLock<String> = OnceLock::new();

/// Set the modules filters
///
/// You need set filters separated by `;`
///
/// Example:
/// ```no_run
/// set_module_filter("wgpu=off;my_super_game=trace");
/// ```
///
/// * `new_filter`: The filter value
pub fn set_module_filter<T: ToString>(new_filter: T) {
    FILTER.set(new_filter.to_string()).unwrap();
}

#[derive(Clone, Debug, Resource)]
pub struct Logs {
    logs: Arc<RwLock<HashMap<LogItem, u64>>>,
    filter_modules: HashMap<String, Option<log::Level>>,
}

impl Default for Logs {
    fn default() -> Self {
        let mut filter_modules = HashMap::new();
        filter_modules.insert(env!("CARGO_PKG_NAME").to_string(), Some(log::Level::Trace));

        let external_filters = FILTER.get_or_init(String::new);
        filter_modules.extend(
            external_filters
                .split(';')
                .flat_map(|s| {
                    let Some((k, v)) = s.split_once('=') else {
                        return None;
                    };
                    Some((
                        k.trim().to_string(),
                        log::Level::from_str(v.trim()).map(Some).unwrap_or(None),
                    ))
                })
                .into_iter(),
        );

        Self {
            logs: Default::default(),
            filter_modules,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct LogItem {
    pub level_log: log::Level,
    pub module: String,
    pub file: String,
    pub line: String,
    pub details: String,
}

impl<'a> From<&log::Record<'a>> for LogItem {
    fn from(v: &log::Record) -> Self {
        Self {
            level_log: v.level(),
            module: v.module_path().unwrap_or_default().to_string(),
            file: v.file().unwrap_or_default().to_string(),
            line: v.line().unwrap_or_default().to_string(),
            details: v.args().to_string(),
        }
    }
}

impl Logs {
    pub fn clear(&self) {
        let mut logs = self.logs.write().unwrap();
        logs.clear();
        drop(logs);
    }

    pub fn len(&self) -> usize {
        let logs = self.logs.read().unwrap();
        let len = logs.len();
        drop(logs);
        len
    }

    pub fn get_logs(&self) -> HashMap<LogItem, u64> {
        let logs = self.logs.read().unwrap();
        let v = logs.clone();
        drop(logs);
        v
    }
}

impl log::Log for Logs {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // Check if NameModule of record has in filter modules and have correct level
        if !record.module_path().is_some_and(|m| {
            self.filter_modules.iter().any(|(name, filter)| {
                m.starts_with(name) && filter.is_some_and(|f| record.level() <= f)
            })
        }) {
            return;
        }

        let mut logs = self.logs.write().unwrap();
        let item = LogItem::from(record);
        (*logs)
            .entry(item)
            .and_modify(|c| {
                if *c <= 99 {
                    *c += 1;
                }
            })
            .or_insert(0);
        drop(logs);
    }

    fn flush(&self) {}
}
