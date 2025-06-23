use crate::module_info::{NodeModuleInfo, SignatureInfo};
use crate::tree_formatter::TreeFormatter;
use anyhow::Result;

/// Trait for different output format visitors
pub trait OutputFormatter {
    /// Format a module tree
    fn format_tree(&self, tree: &NodeModuleInfo) -> Result<String>;

    /// Format a function signature
    fn format_signature(&self, signature: &SignatureInfo) -> Result<String>;

    /// Format a signature not available message
    fn format_signature_not_available(&self, object_name: &str) -> String;
}

/// Pretty print formatter (current default behavior)
pub struct PrettyPrintFormatter {
    tree_formatter: TreeFormatter,
}

impl PrettyPrintFormatter {
    pub fn new() -> Self {
        Self {
            tree_formatter: TreeFormatter::new(),
        }
    }
}

impl Default for PrettyPrintFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for PrettyPrintFormatter {
    fn format_tree(&self, tree: &NodeModuleInfo) -> Result<String> {
        Ok(self.tree_formatter.format_tree(tree))
    }

    fn format_signature(&self, signature: &SignatureInfo) -> Result<String> {
        Ok(self.tree_formatter.format_signature(signature))
    }

    fn format_signature_not_available(&self, object_name: &str) -> String {
        format!("📎 {}\nsignature not available", object_name)
    }
}

/// JSON formatter for machine-readable output
pub struct JsonFormatter;

impl JsonFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for JsonFormatter {
    fn format_tree(&self, tree: &NodeModuleInfo) -> Result<String> {
        Ok(serde_json::to_string_pretty(tree)?)
    }

    fn format_signature(&self, signature: &SignatureInfo) -> Result<String> {
        Ok(serde_json::to_string_pretty(signature)?)
    }

    fn format_signature_not_available(&self, object_name: &str) -> String {
        let fallback = serde_json::json!({
            "name": object_name,
            "kind": "Function",
            "parameters": [],
            "return_type": null,
            "doc_comment": "signature not available"
        });
        serde_json::to_string_pretty(&fallback).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Factory function to create formatter based on format string
pub fn create_formatter(format: &str) -> Box<dyn OutputFormatter> {
    match format.to_lowercase().as_str() {
        "json" => Box::new(JsonFormatter::new()),
        _ => Box::new(PrettyPrintFormatter::new()),
    }
}