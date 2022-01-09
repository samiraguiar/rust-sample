use log::Level;
use log::Metadata;
use log::Record;
use log::SetLoggerError;
use std::io::Write;
use std::time::SystemTime;

struct ConsoleLogger {
    pid: u32,
    prog_name: String,
}

impl log::Log for ConsoleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        eprint!(
            "{:.4?} - {}[{}]: {} - {}\n",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
            self.prog_name,
            self.pid,
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {
        std::io::stderr().flush().expect("Failed to flush stderr!");
    }
}

/// Initialize the console logger as a custom logger.
pub fn init_logger(prog_name: &str, level: Option<Level>) -> Result<(), SetLoggerError> {
    let level = level.unwrap_or(Level::Info);

    let console_logger = Box::new(ConsoleLogger {
        pid: std::process::id(),
        prog_name: prog_name.into(),
    });

    // Register all loggers.
    log::set_boxed_logger(Box::new(console_logger))
        .map(|()| log::set_max_level(level.to_level_filter()))?;

    Ok(())
}
