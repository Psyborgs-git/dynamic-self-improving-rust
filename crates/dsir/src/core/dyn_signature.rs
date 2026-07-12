//! Runtime-configurable signatures (JSON schema + string DSL).

use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use bamltype::baml_types::{StreamingMode, TypeIR, type_meta};
use bamltype::internal_baml_jinja::types::{Class, Name, OutputFormatContent};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::schema::{FieldPath, FieldSchema, InputRenderSpec, SignatureSchema};
use crate::ConstraintSpec;

/// Runtime field type for dynamic signatures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DynFieldType {
    /// Shorthand: `"string"`, `"int"`, `"float"`, `"bool"`.
    Primitive(DynPrimitive),
    /// Explicit object form: `{ "type": "object", "fields": { ... } }`.
    Object {
        #[serde(rename = "type")]
        kind: ObjectTag,
        fields: IndexMap<String, DynFieldType>,
    },
    /// Explicit list form: `{ "type": "list", "items": ... }`.
    List {
        #[serde(rename = "type")]
        kind: ListTag,
        items: Box<DynFieldType>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DynPrimitive {
    String,
    Int,
    Float,
    Bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectTag {
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListTag {
    List,
}

impl DynFieldType {
    pub fn string() -> Self {
        Self::Primitive(DynPrimitive::String)
    }

    pub fn int() -> Self {
        Self::Primitive(DynPrimitive::Int)
    }

    pub fn float() -> Self {
        Self::Primitive(DynPrimitive::Float)
    }

    pub fn bool() -> Self {
        Self::Primitive(DynPrimitive::Bool)
    }

    pub fn object(fields: IndexMap<String, DynFieldType>) -> Self {
        Self::Object {
            kind: ObjectTag::Object,
            fields,
        }
    }

    pub fn list(items: DynFieldType) -> Self {
        Self::List {
            kind: ListTag::List,
            items: Box::new(items),
        }
    }

    fn to_type_ir(&self, class_prefix: &str, classes: &mut Vec<Class>) -> TypeIR {
        match self {
            Self::Primitive(DynPrimitive::String) => TypeIR::string(),
            Self::Primitive(DynPrimitive::Int) => TypeIR::int(),
            Self::Primitive(DynPrimitive::Float) => TypeIR::float(),
            Self::Primitive(DynPrimitive::Bool) => TypeIR::bool(),
            Self::List { items, .. } => {
                TypeIR::list(items.to_type_ir(&format!("{class_prefix}Item"), classes))
            }
            Self::Object { fields, .. } => {
                let class_name = class_prefix.to_string();
                let mut class_fields = Vec::new();
                for (name, ty) in fields {
                    let nested_prefix = format!("{class_name}_{name}");
                    let type_ir = ty.to_type_ir(&nested_prefix, classes);
                    class_fields.push((Name::new(name.clone()), type_ir, None, false));
                }
                classes.push(Class {
                    name: Name::new(class_name.clone()),
                    description: None,
                    namespace: StreamingMode::NonStreaming,
                    fields: class_fields,
                    constraints: Vec::new(),
                    streaming_behavior: type_meta::base::StreamingBehavior::default(),
                });
                TypeIR::class(class_name)
            }
        }
    }
}

/// JSON document describing a dynamic signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynSignatureDoc {
    pub name: String,
    #[serde(default)]
    pub instruction: String,
    #[serde(default)]
    pub inputs: IndexMap<String, DynFieldType>,
    #[serde(default)]
    pub outputs: IndexMap<String, DynFieldType>,
}

/// A runtime signature: instruction + field schemas usable by adapters/optimizers.
#[derive(Debug, Clone)]
pub struct DynSignature {
    pub name: String,
    instruction: String,
    schema: Arc<SignatureSchema>,
    input_keys: Vec<String>,
    output_keys: Vec<String>,
    input_types: IndexMap<String, DynFieldType>,
    output_types: IndexMap<String, DynFieldType>,
}

impl DynSignature {
    pub fn builder() -> DynSignatureBuilder {
        DynSignatureBuilder::default()
    }

    pub fn from_json_str(json: &str) -> Result<Self> {
        let doc: DynSignatureDoc =
            serde_json::from_str(json).context("parse DynSignature JSON")?;
        Self::from_doc(doc)
    }

    pub fn from_json_value(value: Value) -> Result<Self> {
        let doc: DynSignatureDoc =
            serde_json::from_value(value).context("parse DynSignature JSON value")?;
        Self::from_doc(doc)
    }

    pub fn from_doc(doc: DynSignatureDoc) -> Result<Self> {
        if doc.name.trim().is_empty() {
            bail!("DynSignature name must be non-empty");
        }
        if doc.outputs.is_empty() {
            bail!(
                "DynSignature `{}` must declare at least one output field",
                doc.name
            );
        }
        Self::build(doc.name, doc.instruction, doc.inputs, doc.outputs)
    }

    /// Parse a flat string DSL: `"question, context -> answer"` or
    /// `"Answer carefully | question -> answer"`.
    ///
    /// All fields are typed as string. Nested types require JSON.
    pub fn from_dsl(dsl: &str) -> Result<Self> {
        let dsl = dsl.trim();
        if dsl.is_empty() {
            bail!("empty signature DSL");
        }

        let (instruction, body) = if let Some((inst, rest)) = dsl.split_once('|') {
            (inst.trim().to_string(), rest.trim())
        } else {
            (String::new(), dsl)
        };

        let Some((inputs_raw, outputs_raw)) = body.split_once("->") else {
            bail!("signature DSL must contain `->` (got `{dsl}`)");
        };

        let inputs = parse_flat_fields(inputs_raw)?;
        let outputs = parse_flat_fields(outputs_raw)?;
        if outputs.is_empty() {
            bail!("signature DSL must declare at least one output field");
        }

        let name = format!(
            "Dsl_{}",
            outputs.keys().cloned().collect::<Vec<_>>().join("_")
        );
        Self::build(name, instruction, inputs, outputs)
    }

    fn build(
        name: String,
        instruction: String,
        inputs: IndexMap<String, DynFieldType>,
        outputs: IndexMap<String, DynFieldType>,
    ) -> Result<Self> {
        if outputs.is_empty() {
            bail!("DynSignature `{name}` must declare at least one output field");
        }
        let instruction_static: &'static str = Box::leak(instruction.clone().into_boxed_str());

        let mut classes = Vec::new();
        let mut input_fields = Vec::new();
        let mut input_keys = Vec::new();
        for (field_name, field_ty) in &inputs {
            input_keys.push(field_name.clone());
            input_fields.push(make_field_schema(
                field_name,
                field_ty,
                &format!("{name}_In_{field_name}"),
                &mut classes,
            )?);
        }

        let mut output_fields = Vec::new();
        let mut output_keys = Vec::new();
        for (field_name, field_ty) in &outputs {
            output_keys.push(field_name.clone());
            output_fields.push(make_field_schema(
                field_name,
                field_ty,
                &format!("{name}_Out_{field_name}"),
                &mut classes,
            )?);
        }

        let output_class_name = format!("{name}Output");
        let mut root_fields = Vec::new();
        for field in &output_fields {
            root_fields.push((
                Name::new(field.lm_name.to_string()),
                field.type_ir.clone(),
                if field.docs.is_empty() {
                    None
                } else {
                    Some(field.docs.clone())
                },
                false,
            ));
        }
        classes.push(Class {
            name: Name::new(output_class_name.clone()),
            description: None,
            namespace: StreamingMode::NonStreaming,
            fields: root_fields,
            constraints: Vec::new(),
            streaming_behavior: type_meta::base::StreamingBehavior::default(),
        });

        let output_format = OutputFormatContent::target(TypeIR::class(output_class_name))
            .classes(classes)
            .build();

        let schema = SignatureSchema::from_dyn_parts(
            instruction_static,
            input_fields,
            output_fields,
            output_format,
        )
        .map_err(|e| anyhow!(e))?;

        Ok(Self {
            name,
            instruction,
            schema: Arc::new(schema),
            input_keys,
            output_keys,
            input_types: inputs,
            output_types: outputs,
        })
    }

    pub fn instruction(&self) -> &str {
        &self.instruction
    }

    pub fn schema(&self) -> &SignatureSchema {
        &self.schema
    }

    pub fn schema_arc(&self) -> Arc<SignatureSchema> {
        Arc::clone(&self.schema)
    }

    pub fn input_keys(&self) -> &[String] {
        &self.input_keys
    }

    pub fn output_keys(&self) -> &[String] {
        &self.output_keys
    }

    pub fn input_types(&self) -> &IndexMap<String, DynFieldType> {
        &self.input_types
    }

    pub fn output_types(&self) -> &IndexMap<String, DynFieldType> {
        &self.output_types
    }

    /// Clone with an extra string output field (used by dyn ChainOfThought).
    pub fn with_extra_string_output(&self, field: &str, docs: &str) -> Result<Self> {
        let mut outputs = self.output_types.clone();
        if outputs.contains_key(field) {
            return Ok(self.clone());
        }
        outputs.insert(field.to_string(), DynFieldType::string());
        let mut instruction = self.instruction.clone();
        if !docs.is_empty() {
            if !instruction.is_empty() {
                instruction.push(' ');
            }
            instruction.push_str(docs);
        }
        Self::build(
            format!("{}_Aug_{field}", self.name),
            instruction,
            self.input_types.clone(),
            outputs,
        )
    }
}

fn parse_flat_fields(raw: &str) -> Result<IndexMap<String, DynFieldType>> {
    let mut out = IndexMap::new();
    for part in raw.split(',') {
        let name = part.trim();
        if name.is_empty() {
            continue;
        }
        if !is_ident(name) {
            bail!("invalid field name `{name}` in signature DSL");
        }
        out.insert(name.to_string(), DynFieldType::string());
    }
    Ok(out)
}

fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    }
}

fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_owned().into_boxed_str())
}

fn make_field_schema(
    field_name: &str,
    field_ty: &DynFieldType,
    class_prefix: &str,
    classes: &mut Vec<Class>,
) -> Result<FieldSchema> {
    let lm_name = leak_str(field_name);
    let type_ir = field_ty.to_type_ir(class_prefix, classes);
    Ok(FieldSchema {
        lm_name,
        rust_name: field_name.to_string(),
        docs: String::new(),
        type_ir,
        shape: None,
        path: FieldPath::new([lm_name]),
        constraints: EMPTY_CONSTRAINTS,
        input_render: InputRenderSpec::Default,
    })
}

static EMPTY_CONSTRAINTS: &[ConstraintSpec] = &[];

/// Builder for [`DynSignature`].
#[derive(Debug, Default)]
pub struct DynSignatureBuilder {
    name: Option<String>,
    instruction: String,
    inputs: IndexMap<String, DynFieldType>,
    outputs: IndexMap<String, DynFieldType>,
}

impl DynSignatureBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn instruction(mut self, instruction: impl Into<String>) -> Self {
        self.instruction = instruction.into();
        self
    }

    pub fn input(mut self, name: impl Into<String>, ty: DynFieldType) -> Self {
        self.inputs.insert(name.into(), ty);
        self
    }

    pub fn output(mut self, name: impl Into<String>, ty: DynFieldType) -> Self {
        self.outputs.insert(name.into(), ty);
        self
    }

    pub fn build(self) -> Result<DynSignature> {
        let name = self
            .name
            .filter(|n| !n.trim().is_empty())
            .ok_or_else(|| anyhow!("DynSignature builder requires a name"))?;
        DynSignature::build(name, self.instruction, self.inputs, self.outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsl_parses_flat_fields() {
        let sig = DynSignature::from_dsl("question, context -> answer").unwrap();
        assert_eq!(
            sig.input_keys(),
            &["question".to_string(), "context".to_string()]
        );
        assert_eq!(sig.output_keys(), &["answer".to_string()]);
        assert_eq!(sig.schema().input_fields().len(), 2);
        assert_eq!(sig.schema().output_fields().len(), 1);
    }

    #[test]
    fn json_nested_object_and_list() {
        let json = r#"{
            "name": "CiteQA",
            "instruction": "Cite sources.",
            "inputs": {
                "question": "string",
                "context": {
                    "type": "object",
                    "fields": {
                        "docs": { "type": "list", "items": "string" }
                    }
                }
            },
            "outputs": {
                "answer": "string",
                "citations": {
                    "type": "list",
                    "items": {
                        "type": "object",
                        "fields": { "id": "int", "quote": "string" }
                    }
                }
            }
        }"#;
        let sig = DynSignature::from_json_str(json).unwrap();
        assert_eq!(sig.name, "CiteQA");
        assert_eq!(sig.input_keys().len(), 2);
        assert_eq!(sig.output_keys().len(), 2);
        assert!(matches!(
            sig.output_types().get("citations").unwrap(),
            DynFieldType::List { .. }
        ));
    }

    #[test]
    fn builder_rejects_empty_outputs() {
        let err = DynSignature::builder()
            .name("Empty")
            .input("q", DynFieldType::string())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("output"));
    }
}
