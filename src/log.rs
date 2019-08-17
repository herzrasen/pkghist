pub fn setup_logging(verbosity: u64) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, _| out.finish(format_args!("{}", message)))
        .level(level_filter_from_verbosity(verbosity))
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

fn level_filter_from_verbosity(verbosity: u64) -> log::LevelFilter {
    match verbosity {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Warn,
        2 => log::LevelFilter::Info,
        3 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_have_log_filter_trace() {
        let level_filter = level_filter_from_verbosity(4);
        assert_eq!(level_filter, log::LevelFilter::Trace)
    }

    #[test]
    fn should_have_log_filter_debug() {
        let level_filter = level_filter_from_verbosity(3);
        assert_eq!(level_filter, log::LevelFilter::Debug)
    }

    #[test]
    fn should_have_log_filter_info() {
        let level_filter = level_filter_from_verbosity(2);
        assert_eq!(level_filter, log::LevelFilter::Info)
    }

    #[test]
    fn should_have_log_filter_warn() {
        let level_filter = level_filter_from_verbosity(1);
        assert_eq!(level_filter, log::LevelFilter::Warn)
    }

    #[test]
    fn should_have_log_filter_off() {
        let level_filter = level_filter_from_verbosity(0);
        assert_eq!(level_filter, log::LevelFilter::Off)
    }

    // we can only test setting up the logger once per instance created
    #[test]
    fn should_enable_loglevel_trace() {
        setup_logging(4).unwrap();
        assert_eq!(log::log_enabled!(log::Level::Trace), true);
        assert_eq!(log::log_enabled!(log::Level::Debug), true);
        assert_eq!(log::log_enabled!(log::Level::Info), true);
        assert_eq!(log::log_enabled!(log::Level::Warn), true)
    }
}
