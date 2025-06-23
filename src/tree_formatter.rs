use crate::config::Config;
use crate::module_info::*;
use colored::*;

pub struct TreeFormatter {
    config: Config,
}

impl Default for TreeFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeFormatter {
    pub fn new() -> Self {
        Self {
            config: Config::new(),
        }
    }

    pub fn format_tree(&self, module: &NodeModuleInfo) -> String {
        let mut output = String::new();
        self.format_module(module, &mut output, "", true);
        output
    }

    pub fn format_signature(&self, signature: &SignatureInfo) -> String {
        let mut output = String::new();

        let icon = if self.config.no_color {
            &self.config.signature_icon
        } else {
            &self.config.signature_icon.bright_cyan().to_string()
        };

        let name = if self.config.no_color {
            signature.name.clone()
        } else {
            signature.name.bright_blue().to_string()
        };

        output.push_str(&format!("{} {}\n", icon, name));

        if !signature.parameters.is_empty() {
            output.push_str("├── Parameters:\n");

            for (i, param) in signature.parameters.iter().enumerate() {
                let is_last = i == signature.parameters.len() - 1;
                let prefix = if is_last { "└── " } else { "├── " };

                let param_str = self.format_parameter(param);
                output.push_str(&format!("{}{}\n", prefix, param_str));
            }
        }

        if let Some(return_type) = &signature.return_type {
            let return_str = if self.config.no_color {
                format!("Returns: {}", return_type)
            } else {
                format!("Returns: {}", return_type.green())
            };
            output.push_str(&format!("└── {}\n", return_str));
        }

        output
    }

    fn format_module(
        &self,
        module: &NodeModuleInfo,
        output: &mut String,
        prefix: &str,
        is_last: bool,
    ) {
        let current_prefix = if prefix.is_empty() {
            ""
        } else if is_last {
            "└── "
        } else {
            "├── "
        };

        let icon = if self.config.no_color {
            &self.config.module_icon
        } else {
            &self.config.module_icon.bright_yellow().to_string()
        };

        let name = if self.config.no_color {
            module.name.clone()
        } else {
            module.name.bright_blue().to_string()
        };

        let version_str = if let Some(version) = &module.version {
            if self.config.no_color {
                format!("@{}", version)
            } else {
                format!("@{}", version.dimmed())
            }
        } else {
            String::new()
        };

        output.push_str(&format!(
            "{}{}{} {}{}\n",
            prefix, current_prefix, icon, name, version_str
        ));

        let child_prefix = if prefix.is_empty() {
            String::new()
        } else if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        // Show exports if any
        if !module.exports.is_empty() {
            let exports_str = module.exports.join(", ");
            let icon = if self.config.no_color {
                &self.config.exports_icon
            } else {
                &self.config.exports_icon.bright_magenta().to_string()
            };

            let exports_display = if self.config.no_color {
                format!("__all__: {}", exports_str)
            } else {
                format!("__all__: {}", exports_str.cyan())
            };

            output.push_str(&format!(
                "{}├── {} {}\n",
                child_prefix, icon, exports_display
            ));
        }

        // Show functions
        if !module.functions.is_empty() {
            let func_names: Vec<String> = module.functions.iter().map(|f| f.name.clone()).collect();
            let functions_str = func_names.join(", ");

            let icon = if self.config.no_color {
                &self.config.function_icon
            } else {
                &self.config.function_icon.bright_green().to_string()
            };

            let functions_display = if self.config.no_color {
                format!("functions: {}", functions_str)
            } else {
                format!("functions: {}", functions_str.green())
            };

            output.push_str(&format!(
                "{}├── {} {}\n",
                child_prefix, icon, functions_display
            ));
        }

        // Show classes
        if !module.classes.is_empty() {
            let class_names: Vec<String> = module.classes.iter().map(|c| c.name.clone()).collect();
            let classes_str = class_names.join(", ");

            let icon = if self.config.no_color {
                &self.config.class_icon
            } else {
                &self.config.class_icon.bright_blue().to_string()
            };

            let classes_display = if self.config.no_color {
                format!("classes: {}", classes_str)
            } else {
                format!("classes: {}", classes_str.blue())
            };

            output.push_str(&format!(
                "{}├── {} {}\n",
                child_prefix, icon, classes_display
            ));
        }

        // Show types
        if !module.types.is_empty() {
            let type_names: Vec<String> = module.types.iter().map(|t| t.name.clone()).collect();
            let types_str = type_names.join(", ");

            let types_display = if self.config.no_color {
                format!("types: {}", types_str)
            } else {
                format!("types: {}", types_str.purple())
            };

            output.push_str(&format!("{}├── 🔷 {}\n", child_prefix, types_display));
        }

        // Show constants
        if !module.constants.is_empty() {
            let const_names: Vec<String> =
                module.constants.iter().map(|c| c.name.clone()).collect();
            let constants_str = const_names.join(", ");

            let icon = if self.config.no_color {
                &self.config.constant_icon
            } else {
                &self.config.constant_icon.bright_red().to_string()
            };

            let constants_display = if self.config.no_color {
                format!("constants: {}", constants_str)
            } else {
                format!("constants: {}", constants_str.red())
            };

            output.push_str(&format!(
                "{}├── {} {}\n",
                child_prefix, icon, constants_display
            ));
        }

        // Show submodules
        let submodule_count = module.submodules.len();
        for (i, (_, submodule)) in module.submodules.iter().enumerate() {
            let is_last_submodule = i == submodule_count - 1;
            self.format_module(submodule, output, &child_prefix, is_last_submodule);
        }
    }

    fn format_parameter(&self, param: &Parameter) -> String {
        let mut param_str = String::new();

        if param.is_rest {
            param_str.push_str("...");
        }

        let name = if self.config.no_color {
            param.name.clone()
        } else {
            param.name.bright_white().to_string()
        };

        param_str.push_str(&name);

        if param.is_optional {
            param_str.push('?');
        }

        if let Some(param_type) = &param.param_type {
            let type_str = if self.config.no_color {
                format!(": {}", param_type)
            } else {
                format!(": {}", param_type.bright_yellow())
            };
            param_str.push_str(&type_str);
        }

        if let Some(default_value) = &param.default_value {
            let default_str = if self.config.no_color {
                format!(" = {}", default_value)
            } else {
                format!(" = {}", default_value.dimmed())
            };
            param_str.push_str(&default_str);
        }

        param_str
    }
}
