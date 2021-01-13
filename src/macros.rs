#[macro_export]
macro_rules! plugin_from {
    ($cfg:expr, $plugin:ident) => {{
        match $cfg.as_ref() {
            Some(cfg) => match cfg.$plugin.as_ref() {
                Some(plg) => Some(plg),
                None => {
                    log::info!(
                        "plugin {} is not defined in config file",
                        stringify!($plugin)
                    );
                    None
                }
            },
            None => {
                log::warn!("no plugins defined for {} in config file", stringify!($cfg));
                None
            }
        }
    }};
}

#[macro_export]
macro_rules! register_plugins {
    ($type:ident => $config:ident.$plugins:ident.$plugin:ident) => {{
        match super::plugin_from!($config.$plugins, $plugin) {
            Some(plgs) => {
                let mut plugins: Vec<Box<dyn $type>> = Vec::new();
                for plg in plgs.iter() {
                    plugins.push(Box::new(plg.clone()));
                }
                let plugin_group = stringify!("{}", $plugin).to_string();
                $plugins.insert(plugin_group, plugins);
            }
            None => println!(""),
        };
    }};
}
