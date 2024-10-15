pub struct Logger {
    cron: bool,
}

impl Logger {
    pub fn new(cron: bool) -> Self {
        if cron {
            let _ = env_logger::try_init().is_ok();
        }

        Self { cron }
    }

    pub fn info(&self, msg: &str) {
        if self.cron {
            log::info!("{msg}");
        } else {
            println!("{msg}");
        }
    }

    pub fn error(&self, msg: &str) {
        if self.cron {
            log::error!("{msg}");
        } else {
            eprintln!("{msg}");
        }
    }
}
