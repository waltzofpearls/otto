#[macro_export]
macro_rules! plugin_from {
    ($cfg:expr, $plugin:ident) => {{
        match $cfg.as_ref() {
            Some(cfg) => match cfg.$plugin.as_ref() {
                Some(plg) => Some(plg),
                None => {
                    log::info!("plugin {} is not defined in config file", stringify!($plugin));
                    None
                },
            },
            None => {
                log::warn!("no plugins defined for {} in config file", stringify!($cfg));
                None
            },
        }
    }};
}
