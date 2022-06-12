use env_logger::filter::{Builder, Filter as EnvFilter};

use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};

const FILTER_ENV: &str = "WEKAN_LOG";

pub struct Logger {
    inner: EnvFilter,
}

impl Logger {
    fn new() -> Logger {
        let mut builder = Builder::from_env(FILTER_ENV);

        Logger {
            inner: builder.build(),
        }
    }

    pub fn init(verbose: bool) -> Result<(), SetLoggerError> {
        let logger = Self::new();
        if verbose {
            log::set_max_level(logger.inner.filter());
        } else {
            log::set_max_level(LevelFilter::Info);
        }
        log::set_boxed_logger(Box::new(logger))
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        // Check if the record is matched by the logger before logging
        if self.inner.matches(record) {
            match std::env::var("WEKAN_BACKTRACE") {
                Ok(external) => {
                    if external.contains('1') || external.contains("true") {
                        Logger::print_by_level(record)
                    } else {
                        panic!("WEKAN_BACKTRACE env is dirty. Let's panic.");
                    }
                }
                Err(_err) => {
                    if record.target().starts_with("wekan") {
                        match std::env::var("WEKAN_LOG_MFILTER") {
                            Ok(m) => {
                                if record.target().ends_with(&m) {
                                    Logger::print_by_level(record);
                                }
                            }
                            Err(_e) => Logger::print_by_level(record),
                        }
                    }
                }
            }
        }
    }

    fn flush(&self) {}
}
trait Filter {
    fn print_by_level(record: &Record);
}
impl Filter for Logger {
    fn print_by_level(record: &Record) {
        match record.level() {
            log::Level::Info => {
                println!(
                    "{} {}-fn -- {}",
                    record.level(),
                    record.target(),
                    record.args()
                );
            }
            log::Level::Debug => {
                println!(
                    "{} {}-fn -- {}",
                    record.level(),
                    record.target(),
                    record.args()
                );
            }
            log::Level::Warn => {
                println!("WARN: {}", record.args());
            }
            log::Level::Error => {
                println!("ERROR: {}", record.args());
            }
            log::Level::Trace => {
                println!(
                    "{} {}:{}-{}",
                    record.level(),
                    record.file().unwrap(),
                    record.line().unwrap(),
                    record.target()
                );
                println!("{} {}", record.level(), record.args());
            }
        }
    }
}
