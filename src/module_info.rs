use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeModuleInfo {
    pub name: String,
    pub version: Option<String>,
    pub main: Option<String>,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub types: Vec<TypeInfo>,
    pub constants: Vec<ConstantInfo>,
    pub submodules: HashMap<String, NodeModuleInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub is_generator: bool,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub name: String,
    pub constructor: Option<FunctionInfo>,
    pub methods: Vec<FunctionInfo>,
    pub properties: Vec<PropertyInfo>,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub name: String,
    pub kind: TypeKind,
    pub definition: String,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    Interface,
    Type,
    Enum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantInfo {
    pub name: String,
    pub value_type: Option<String>,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyInfo {
    pub name: String,
    pub property_type: Option<String>,
    pub is_readonly: bool,
    pub is_static: bool,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<String>,
    pub is_optional: bool,
    pub is_rest: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub name: String,
    pub kind: SignatureKind,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureKind {
    Function,
    Method,
    Constructor,
    ArrowFunction,
}

impl NodeModuleInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: None,
            main: None,
            exports: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            classes: Vec::new(),
            types: Vec::new(),
            constants: Vec::new(),
            submodules: HashMap::new(),
        }
    }

    pub fn add_submodule(&mut self, name: String, module: NodeModuleInfo) {
        self.submodules.insert(name, module);
    }

    pub fn add_function(&mut self, function: FunctionInfo) {
        self.functions.push(function);
    }

    pub fn add_class(&mut self, class: ClassInfo) {
        self.classes.push(class);
    }

    pub fn add_type(&mut self, type_info: TypeInfo) {
        self.types.push(type_info);
    }

    pub fn add_constant(&mut self, constant: ConstantInfo) {
        self.constants.push(constant);
    }
}
