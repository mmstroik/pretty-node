use crate::module_info::*;
// use crate::parser::semantic_analyzer::SemanticAnalyzer; // TODO: Fix compilation errors
use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};

pub struct AstParser {
    source_map: Lrc<SourceMap>,
}

impl Default for AstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AstParser {
    pub fn new() -> Self {
        Self {
            source_map: Lrc::new(SourceMap::default()),
        }
    }

    /// Parse a JavaScript/TypeScript file and extract module information
    pub fn parse_file(&self, file_path: &Path) -> Result<NodeModuleInfo> {
        let content = fs::read_to_string(file_path)?;
        let module_name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        let module_info = self.parse_content(&content, module_name)?;
        
        // TODO: Add enhanced semantic analysis for better symbol extraction
        // let mut analyzer = SemanticAnalyzer::new();
        // if analyzer.analyze_file(file_path).is_ok() {
        //     // Extract additional info using semantic analysis
        //     if analyzer.extract_module_info(&mut module_info).is_ok() {
        //         // Semantic analysis succeeded - we now have method signatures too
        //     }
        // }
        
        Ok(module_info)
    }

    /// Parse JavaScript/TypeScript content and extract module information
    pub fn parse_content(&self, content: &str, module_name: &str) -> Result<NodeModuleInfo> {
        let source_file = self
            .source_map
            .new_source_file(FileName::Anon, content.to_string());

        let lexer = Lexer::new(
            Syntax::Typescript(TsConfig {
                tsx: true,
                decorators: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*source_file),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        let module = parser
            .parse_module()
            .map_err(|e| anyhow!("Parse error: {:?}", e))?;

        let mut module_info = NodeModuleInfo::new(module_name.to_string());

        for item in &module.body {
            self.process_module_item(item, &mut module_info)?;
        }

        Ok(module_info)
    }

    fn process_module_item(
        &self,
        item: &ModuleItem,
        module_info: &mut NodeModuleInfo,
    ) -> Result<()> {
        match item {
            ModuleItem::ModuleDecl(decl) => {
                self.process_module_decl(decl, module_info)?;
            }
            ModuleItem::Stmt(stmt) => {
                self.process_stmt(stmt, module_info)?;
            }
        }
        Ok(())
    }

    fn process_module_decl(
        &self,
        decl: &ModuleDecl,
        module_info: &mut NodeModuleInfo,
    ) -> Result<()> {
        match decl {
            ModuleDecl::ExportDecl(export_decl) => {
                self.process_decl(&export_decl.decl, module_info, true)?;
            }
            ModuleDecl::ExportNamed(named_export) => {
                for spec in &named_export.specifiers {
                    if let ExportSpecifier::Named(named) = spec {
                        let name = match &named.exported {
                            Some(exported) => self.module_export_name_to_string(exported),
                            None => self.module_export_name_to_string(&named.orig),
                        };
                        module_info.exports.push(name);
                    }
                }
            }
            ModuleDecl::ExportDefaultDecl(default_export) => {
                match &default_export.decl {
                    DefaultDecl::Class(class_expr) => {
                        if let Some(ident) = &class_expr.ident {
                            let class_info = self.extract_class_info(&class_expr.class, &ident.sym);
                            module_info.add_class(class_info);
                        }
                    }
                    DefaultDecl::Fn(fn_expr) => {
                        if let Some(ident) = &fn_expr.ident {
                            let func_info =
                                self.extract_function_info(&fn_expr.function, &ident.sym);
                            module_info.add_function(func_info);
                        }
                    }
                    _ => {}
                }
                module_info.exports.push("default".to_string());
            }
            _ => {}
        }
        Ok(())
    }

    fn process_stmt(&self, stmt: &Stmt, module_info: &mut NodeModuleInfo) -> Result<()> {
        if let Stmt::Decl(decl) = stmt {
            self.process_decl(decl, module_info, false)?;
        }
        Ok(())
    }

    fn process_decl(
        &self,
        decl: &Decl,
        module_info: &mut NodeModuleInfo,
        is_export: bool,
    ) -> Result<()> {
        match decl {
            Decl::Fn(fn_decl) => {
                let func_info = self.extract_function_info(&fn_decl.function, &fn_decl.ident.sym);
                if is_export {
                    module_info.exports.push(fn_decl.ident.sym.to_string());
                }
                module_info.add_function(func_info);
            }
            Decl::Class(class_decl) => {
                let class_info = self.extract_class_info(&class_decl.class, &class_decl.ident.sym);
                if is_export {
                    module_info.exports.push(class_decl.ident.sym.to_string());
                }
                module_info.add_class(class_info);
            }
            Decl::Var(var_decl) => {
                for decl in &var_decl.decls {
                    if let Pat::Ident(ident) = &decl.name {
                        let name = ident.id.sym.to_string();
                        if is_export {
                            module_info.exports.push(name.clone());
                        }

                        // Try to determine if this is a function or constant
                        if let Some(init) = &decl.init {
                            match &**init {
                                Expr::Arrow(_) | Expr::Fn(_) => {
                                    // It's a function
                                    let func_info = FunctionInfo {
                                        name: name.clone(),
                                        parameters: Vec::new(), // TODO: extract from arrow/fn
                                        return_type: None,
                                        is_async: false,
                                        is_generator: false,
                                        doc_comment: None,
                                    };
                                    module_info.add_function(func_info);
                                }
                                _ => {
                                    // It's a constant
                                    let const_info = ConstantInfo {
                                        name: name.clone(),
                                        value_type: None, // TODO: infer type
                                        doc_comment: None,
                                    };
                                    module_info.add_constant(const_info);
                                }
                            }
                        }
                    }
                }
            }
            Decl::TsInterface(interface_decl) => {
                let type_info = TypeInfo {
                    name: interface_decl.id.sym.to_string(),
                    kind: TypeKind::Interface,
                    definition: format!("interface {}", interface_decl.id.sym),
                    doc_comment: None,
                };
                if is_export {
                    module_info.exports.push(interface_decl.id.sym.to_string());
                }
                module_info.add_type(type_info);
            }
            Decl::TsTypeAlias(type_alias) => {
                let type_info = TypeInfo {
                    name: type_alias.id.sym.to_string(),
                    kind: TypeKind::Type,
                    definition: format!("type {}", type_alias.id.sym),
                    doc_comment: None,
                };
                if is_export {
                    module_info.exports.push(type_alias.id.sym.to_string());
                }
                module_info.add_type(type_info);
            }
            Decl::TsEnum(enum_decl) => {
                let type_info = TypeInfo {
                    name: enum_decl.id.sym.to_string(),
                    kind: TypeKind::Enum,
                    definition: format!("enum {}", enum_decl.id.sym),
                    doc_comment: None,
                };
                if is_export {
                    module_info.exports.push(enum_decl.id.sym.to_string());
                }
                module_info.add_type(type_info);
            }
            _ => {}
        }
        Ok(())
    }

    fn extract_function_info(&self, function: &Function, name: &str) -> FunctionInfo {
        let parameters = function
            .params
            .iter()
            .map(|param| self.extract_parameter_info(&param.pat))
            .collect();

        FunctionInfo {
            name: name.to_string(),
            parameters,
            return_type: function.return_type.as_ref().map(|_| "unknown".to_string()), // TODO: extract actual type
            is_async: function.is_async,
            is_generator: function.is_generator,
            doc_comment: None,
        }
    }

    fn extract_class_info(&self, class: &Class, name: &str) -> ClassInfo {
        let mut methods = Vec::new();
        let mut properties = Vec::new();
        let mut constructor = None;

        for member in &class.body {
            match member {
                ClassMember::Constructor(ctor) => {
                    let func_info = FunctionInfo {
                        name: "constructor".to_string(),
                        parameters: ctor
                            .params
                            .iter()
                            .filter_map(|param| match param {
                                ParamOrTsParamProp::Param(p) => {
                                    Some(self.extract_parameter_info(&p.pat))
                                }
                                _ => None,
                            })
                            .collect(),
                        return_type: None,
                        is_async: false,
                        is_generator: false,
                        doc_comment: None,
                    };
                    constructor = Some(func_info);
                }
                ClassMember::Method(method) => {
                    if let PropName::Ident(ident) = &method.key {
                        let func_info = self.extract_function_info(&method.function, &ident.sym);
                        methods.push(func_info);
                    }
                }
                ClassMember::ClassProp(prop) => {
                    if let PropName::Ident(ident) = &prop.key {
                        let prop_info = PropertyInfo {
                            name: ident.sym.to_string(),
                            property_type: prop.type_ann.as_ref().map(|_| "unknown".to_string()),
                            is_readonly: prop.readonly,
                            is_static: prop.is_static,
                            doc_comment: None,
                        };
                        properties.push(prop_info);
                    }
                }
                _ => {}
            }
        }

        ClassInfo {
            name: name.to_string(),
            constructor,
            methods,
            properties,
            extends: class.super_class.as_ref().map(|_| "unknown".to_string()), // TODO: extract actual parent
            implements: Vec::new(), // TODO: extract implements
            doc_comment: None,
        }
    }

    fn extract_parameter_info(&self, pat: &Pat) -> Parameter {
        match pat {
            Pat::Ident(ident) => Parameter {
                name: ident.id.sym.to_string(),
                param_type: ident.type_ann.as_ref().map(|_| "unknown".to_string()),
                is_optional: ident.optional,
                is_rest: false,
                default_value: None,
            },
            Pat::Rest(rest) => {
                if let Pat::Ident(ident) = &*rest.arg {
                    Parameter {
                        name: ident.id.sym.to_string(),
                        param_type: ident.type_ann.as_ref().map(|_| "unknown".to_string()),
                        is_optional: false,
                        is_rest: true,
                        default_value: None,
                    }
                } else {
                    Parameter {
                        name: "...unknown".to_string(),
                        param_type: None,
                        is_optional: false,
                        is_rest: true,
                        default_value: None,
                    }
                }
            }
            Pat::Assign(assign) => {
                if let Pat::Ident(ident) = &*assign.left {
                    Parameter {
                        name: ident.id.sym.to_string(),
                        param_type: ident.type_ann.as_ref().map(|_| "unknown".to_string()),
                        is_optional: true,
                        is_rest: false,
                        default_value: Some("default".to_string()), // TODO: extract actual default
                    }
                } else {
                    Parameter {
                        name: "unknown".to_string(),
                        param_type: None,
                        is_optional: true,
                        is_rest: false,
                        default_value: Some("default".to_string()),
                    }
                }
            }
            _ => Parameter {
                name: "unknown".to_string(),
                param_type: None,
                is_optional: false,
                is_rest: false,
                default_value: None,
            },
        }
    }

    #[allow(dead_code)]
    fn ident_to_string(&self, ident: &Ident) -> String {
        ident.sym.to_string()
    }

    fn module_export_name_to_string(&self, name: &ModuleExportName) -> String {
        match name {
            ModuleExportName::Ident(ident) => ident.sym.to_string(),
            ModuleExportName::Str(s) => s.value.to_string(),
        }
    }
}
