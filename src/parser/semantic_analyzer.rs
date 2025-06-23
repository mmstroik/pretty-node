use crate::module_info::{NodeModuleInfo, FunctionInfo, ClassInfo, Parameter, PropertyInfo};
use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use swc_common::{SourceMap, sync::Lrc};
use swc_ecma_ast::*;
use swc_ecma_parser::{Parser, StringInput, Syntax, TsConfig};
use swc_ecma_visit::{Visit, VisitWith};

macro_rules! debug_log {
    ($($arg:tt)*) => {
        if env::var("PRETTY_NODE_DEBUG").is_ok() {
            eprintln!("[DEBUG] {}", format!($($arg)*));
        }
    };
}

/// Enhanced semantic analysis using swc's AST visitor pattern
/// This approach provides deeper analysis than basic parsing
pub struct SemanticAnalyzer {
    /// Track the current scope stack to classify functions vs methods
    scope_stack: Vec<ScopeContext>,
    /// Map of function/method signatures found
    signatures: HashMap<String, FunctionInfo>,
    /// Map of class definitions found
    classes: HashMap<String, ClassInfo>,
    /// Current class being analyzed
    current_class: Option<String>,
}

#[derive(Debug, Clone)]
enum ScopeContext {
    Module,
    Class(String),
    Function(String),
    Method(String, String), // class_name, method_name
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scope_stack: vec![ScopeContext::Module],
            signatures: HashMap::new(),
            classes: HashMap::new(),
            current_class: None,
        }
    }

    /// Analyze a JavaScript/TypeScript file using AST visitor pattern
    pub fn analyze_file(&mut self, file_path: &Path) -> Result<()> {
        let source_code = std::fs::read_to_string(file_path)?;
        
        // Set up swc parser
        let cm: Lrc<SourceMap> = Default::default();
        let syntax = if file_path.extension().map_or(false, |ext| ext == "ts" || ext == "d.ts") {
            Syntax::Typescript(TsConfig {
                tsx: false,
                decorators: true,
                dts: file_path.extension().map_or(false, |ext| ext == "d.ts"),
                no_early_errors: true,
                disallow_ambiguous_jsx_like: false,
            })
        } else {
            Syntax::Es(Default::default())
        };
        
        let input = StringInput::new(
            &source_code,
            swc_common::BytePos(0),
            swc_common::BytePos(source_code.len() as u32),
        );
        
        let mut parser = Parser::new(syntax, input, None);
        
        // Parse the module
        match parser.parse_module() {
            Ok(module) => {
                debug_log!("Successfully parsed module: {:?}", file_path);
                module.visit_with(self);
                Ok(())
            }
            Err(e) => {
                debug_log!("Failed to parse {}: {:?}", file_path.display(), e);
                Err(anyhow::anyhow!("Parse error: {:?}", e))
            }
        }
    }

    /// Extract enhanced module info with method classification
    pub fn extract_module_info(&self, base_info: &mut NodeModuleInfo) -> Result<()> {
        // Add all function signatures we found
        for (name, function_info) in &self.signatures {
            if !base_info.functions.iter().any(|f| f.name == *name) {
                base_info.functions.push(function_info.clone());
            }
        }

        // Add all class info we found
        for (name, class_info) in &self.classes {
            if !base_info.classes.iter().any(|c| c.name == *name) {
                base_info.classes.push(class_info.clone());
            }
        }

        Ok(())
    }

    /// Check if we're currently in a class scope
    fn in_class_scope(&self) -> Option<&String> {
        for scope in self.scope_stack.iter().rev() {
            if let ScopeContext::Class(class_name) = scope {
                return Some(class_name);
            }
        }
        None
    }

    /// Extract parameters from function/method
    fn extract_parameters(&self, params: &[Param]) -> Vec<Parameter> {
        params
            .iter()
            .filter_map(|param| match &param.pat {
                Pat::Ident(ident) => Some(Parameter {
                    name: ident.id.sym.to_string(),
                    param_type: None, // TODO: Extract type annotations
                    is_optional: false,
                    is_rest: false,
                    default_value: None,
                }),
                _ => None,
            })
            .collect()
    }

    /// Extract return type from function
    fn extract_return_type(&self, return_type: &Option<Box<TsTypeAnn>>) -> Option<String> {
        return_type.as_ref().map(|_| "any".to_string()) // TODO: Parse actual types
    }
}

/// Custom visitor implementation for semantic analysis
impl Visit for SemanticAnalyzer {
    fn visit_function_decl(&mut self, func: &FunctionDecl) {
        let function_name = func.ident.sym.to_string();
        debug_log!("Found function: {}", function_name);

        let parameters = self.extract_parameters(&func.function.params);
        let return_type = self.extract_return_type(&func.function.return_type);

        let function_info = FunctionInfo {
            name: function_name.clone(),
            parameters,
            return_type,
            is_async: func.function.is_async,
            is_generator: func.function.is_generator,
            doc_comment: None, // TODO: Extract JSDoc comments
        };

        if let Some(class_name) = self.in_class_scope().cloned() {
            // This is a method - store in the current class
            if let Some(class_info) = self.classes.get_mut(&class_name) {
                class_info.methods.push(function_info);
            }
        } else {
            // This is a module-level function
            self.signatures.insert(function_name, function_info);
        }

        // Enter function scope
        self.scope_stack.push(ScopeContext::Function(func.ident.sym.to_string()));

        // Visit function body
        func.function.visit_with(self);

        // Exit function scope
        self.scope_stack.pop();
    }

    fn visit_class_decl(&mut self, class: &ClassDecl) {
        let class_name = class.ident.sym.to_string();
        debug_log!("Found class: {}", class_name);

        let class_info = ClassInfo {
            name: class_name.clone(),
            constructor: None, // Will be filled when we visit constructor
            methods: Vec::new(),
            properties: Vec::new(),
            doc_comment: None, // TODO: Extract JSDoc comments
        };

        self.classes.insert(class_name.clone(), class_info);
        self.current_class = Some(class_name.clone());

        // Enter class scope
        self.scope_stack.push(ScopeContext::Class(class_name));

        // Visit class body
        class.class.visit_with(self);

        // Exit class scope
        self.scope_stack.pop();
        self.current_class = None;
    }

    fn visit_constructor(&mut self, constructor: &Constructor) {
        debug_log!("Found constructor");

        if let Some(class_name) = &self.current_class {
            let parameters = self.extract_parameters(&constructor.params);
            
            if let Some(class_info) = self.classes.get_mut(class_name) {
                class_info.constructor = Some(FunctionInfo {
                    name: "constructor".to_string(),
                    parameters,
                    return_type: Some(class_name.clone()),
                    is_async: false,
                    is_generator: false,
                    doc_comment: None,
                });
            }
        }

        // Visit constructor body
        constructor.visit_with(self);
    }

    fn visit_class_method(&mut self, method: &ClassMethod) {
        if let PropName::Ident(ident) = &method.key {
            let method_name = ident.sym.to_string();
            debug_log!("Found class method: {}", method_name);

            if let Some(class_name) = &self.current_class {
                let parameters = self.extract_parameters(&method.function.params);
                let return_type = self.extract_return_type(&method.function.return_type);
                let is_static = method.is_static;

                let method_info = FunctionInfo {
                    name: method_name.clone(),
                    parameters,
                    return_type,
                    is_async: method.function.is_async,
                    is_generator: method.function.is_generator,
                    doc_comment: None,
                };

                if let Some(class_info) = self.classes.get_mut(class_name) {
                    class_info.methods.push(method_info);
                }

                // Enter method scope
                self.scope_stack.push(ScopeContext::Method(class_name.clone(), method_name));

                // Visit method body
                method.function.visit_with(self);

                // Exit method scope
                self.scope_stack.pop();
            }
        }
    }

    fn visit_export_named_decl(&mut self, export: &NamedExport) {
        debug_log!("Found named export");
        // TODO: Track exports for better symbol resolution
        export.visit_with(self);
    }

    fn visit_export_default_decl(&mut self, export: &ExportDefaultDecl) {
        debug_log!("Found default export");
        // TODO: Track default exports
        export.visit_with(self);
    }

    fn visit_import_decl(&mut self, import: &ImportDecl) {
        debug_log!("Found import from: {}", import.src.value);
        // TODO: Track imports for better symbol resolution
        import.visit_with(self);
    }
}