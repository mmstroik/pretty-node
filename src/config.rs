use std::env;

pub struct Config {
    pub no_color: bool,
    pub module_icon: String,
    pub function_icon: String,
    pub class_icon: String,
    pub constant_icon: String,
    pub exports_icon: String,
    pub signature_icon: String,
}

impl Default for Config {
    fn default() -> Self {
        let no_color = env::var("NO_COLOR").is_ok() || env::var("PRETTY_NODE_NO_COLOR").is_ok();
        let ascii_mode = env::var("PRETTY_NODE_ASCII").is_ok();

        Self {
            no_color,
            module_icon: env::var("PRETTY_NODE_MODULE_ICON").unwrap_or_else(|_| {
                if ascii_mode {
                    "[M]".to_string()
                } else {
                    "📦".to_string()
                }
            }),
            function_icon: env::var("PRETTY_NODE_FUNCTION_ICON").unwrap_or_else(|_| {
                if ascii_mode {
                    "fn".to_string()
                } else {
                    "⚡".to_string()
                }
            }),
            class_icon: env::var("PRETTY_NODE_CLASS_ICON").unwrap_or_else(|_| {
                if ascii_mode {
                    "cls".to_string()
                } else {
                    "🔷".to_string()
                }
            }),
            constant_icon: env::var("PRETTY_NODE_CONSTANT_ICON").unwrap_or_else(|_| {
                if ascii_mode {
                    "const".to_string()
                } else {
                    "📌".to_string()
                }
            }),
            exports_icon: env::var("PRETTY_NODE_EXPORTS_ICON").unwrap_or_else(|_| {
                if ascii_mode {
                    "exp".to_string()
                } else {
                    "📜".to_string()
                }
            }),
            signature_icon: env::var("PRETTY_NODE_SIGNATURE_ICON").unwrap_or_else(|_| {
                if ascii_mode {
                    "sig".to_string()
                } else {
                    "📎".to_string()
                }
            }),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
