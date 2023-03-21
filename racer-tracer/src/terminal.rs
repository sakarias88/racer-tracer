use console::Term;
use slog::Logger;

pub struct Terminal {
    pub logger: Logger,
    pub terminal: Term,
}

impl Terminal {
    pub fn new(logger: Logger) -> Self {
        Self {
            logger,
            terminal: Term::stdout(),
        }
    }
}

macro_rules! write_term {
    ($term:expr, $text:expr) => {{
        if let Err(e) = $term.terminal.write_line($text) {
            debug!($term.logger, "Failed to write to terminal: {}", e)
        }
    }};
}

pub(crate) use write_term;
