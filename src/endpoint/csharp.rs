//! Deterministic C# emitter over the endpoint protocol's JSON Schema.
//!
//! Supports exactly the shapes the protocol uses and fails loudly on anything
//! else, so a new wire shape cannot silently produce wrong C#.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde_json::Value;

use super::protocol::{DownlinkFrame, UplinkFrame, PROTOCOL_VERSION};

pub const NAMESPACE: &str = "Opr8r.Endpoint.Protocol";

const DISCRIMINATOR: &str = "type";
const INDENT: &str = "    ";

#[derive(Debug, PartialEq, Eq)]
pub struct EmitError(pub String);

fn err<T>(msg: impl Into<String>) -> Result<T, EmitError> {
    Err(EmitError(msg.into()))
}

fn pascal(snake: &str) -> String {
    snake
        .split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            c.next()
                .map(|f| f.to_uppercase().chain(c).collect::<String>())
                .unwrap_or_default()
        })
        .collect()
}

fn ref_name(schema: &Value) -> Option<&str> {
    schema
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|r| r.strip_prefix("#/$defs/"))
}

fn integer_type(format: Option<&str>) -> Result<&'static str, EmitError> {
    match format {
        Some("int32") => Ok("int"),
        Some("int64") => Ok("long"),
        Some("uint16") => Ok("ushort"),
        Some("uint32") => Ok("uint"),
        Some("uint64") => Ok("ulong"),
        other => err(format!("unsupported integer format {other:?}")),
    }
}

fn scalar_type(ty: &str, schema: &Value) -> Result<String, EmitError> {
    match ty {
        "string" => Ok("string".into()),
        "boolean" => Ok("bool".into()),
        "integer" => integer_type(schema.get("format").and_then(Value::as_str)).map(Into::into),
        "array" => {
            let items = schema
                .get("items")
                .ok_or_else(|| EmitError("array without items".into()))?;
            Ok(format!("IReadOnlyList<{}>", field_type(items)?.ty))
        }
        "object" => {
            let values = schema
                .get("additionalProperties")
                .filter(|v| v.is_object())
                .ok_or_else(|| EmitError("object field without additionalProperties".into()))?;
            Ok(format!(
                "IReadOnlyDictionary<string, {}>",
                field_type(values)?.ty
            ))
        }
        other => err(format!("unsupported JSON type {other:?}")),
    }
}

struct FieldType {
    ty: String,
    nullable: bool,
}

fn field_type(schema: &Value) -> Result<FieldType, EmitError> {
    if let Some(name) = ref_name(schema) {
        return Ok(FieldType {
            ty: name.into(),
            nullable: false,
        });
    }
    if let Some(any_of) = schema.get("anyOf").and_then(Value::as_array) {
        let non_null: Vec<&Value> = any_of
            .iter()
            .filter(|s| s.get("type").and_then(Value::as_str) != Some("null"))
            .collect();
        return match non_null.as_slice() {
            [single] if any_of.len() == 2 => Ok(FieldType {
                ty: field_type(single)?.ty,
                nullable: true,
            }),
            _ => err("anyOf other than T | null is unsupported"),
        };
    }
    match schema.get("type") {
        Some(Value::String(ty)) => Ok(FieldType {
            ty: scalar_type(ty, schema)?,
            nullable: false,
        }),
        Some(Value::Array(types)) => {
            let non_null: Vec<&str> = types
                .iter()
                .filter_map(Value::as_str)
                .filter(|t| *t != "null")
                .collect();
            match non_null.as_slice() {
                [ty] if types.len() == 2 => Ok(FieldType {
                    ty: scalar_type(ty, schema)?,
                    nullable: true,
                }),
                _ => err(format!("unsupported type union {types:?}")),
            }
        }
        _ => err(format!("field schema without type: {schema}")),
    }
}

fn emit_enum(out: &mut String, name: &str, variants: &[Value]) -> Result<(), EmitError> {
    writeln!(out, "[JsonConverter(typeof(JsonStringEnumConverter<{name}>))]").ok();
    writeln!(out, "public enum {name}\n{{").ok();
    for v in variants {
        let wire = v
            .as_str()
            .ok_or_else(|| EmitError(format!("non-string enum value in {name}")))?;
        writeln!(out, "{INDENT}[JsonStringEnumMemberName(\"{wire}\")]").ok();
        writeln!(out, "{INDENT}{},", pascal(wire)).ok();
    }
    writeln!(out, "}}\n").ok();
    Ok(())
}

fn emit_record(
    out: &mut String,
    name: &str,
    schema: &Value,
    base: Option<&str>,
) -> Result<(), EmitError> {
    let required: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|r| r.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let inherits = base.map(|b| format!(" : {b}")).unwrap_or_default();
    writeln!(out, "public sealed record {name}{inherits}\n{{").ok();
    let empty = serde_json::Map::new();
    let props = schema
        .get("properties")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    for (wire, prop) in props {
        let FieldType { ty, nullable } = field_type(prop)?;
        let member = pascal(wire);
        let is_required = required.contains(&wire.as_str());
        let decl = if nullable {
            format!("public {ty}? {member} {{ get; init; }}")
        } else if is_required {
            format!("public required {ty} {member} {{ get; init; }}")
        } else if ty.starts_with("IReadOnlyList<") {
            format!("public {ty} {member} {{ get; init; }} = [];")
        } else if let Some(args) = ty.strip_prefix("IReadOnlyDictionary<") {
            format!("public {ty} {member} {{ get; init; }} = new Dictionary<{args}();")
        } else {
            return err(format!(
                "{name}.{wire}: optional non-collection fields must be nullable"
            ));
        };
        writeln!(out, "{INDENT}[JsonPropertyName(\"{wire}\")]").ok();
        writeln!(out, "{INDENT}{decl}").ok();
    }
    writeln!(out, "}}\n").ok();
    Ok(())
}

fn frame_variants(frame: &str, root: &Value) -> Result<Vec<(String, String)>, EmitError> {
    let one_of = root
        .get("oneOf")
        .and_then(Value::as_array)
        .ok_or_else(|| EmitError(format!("{frame} is not a oneOf")))?;
    one_of
        .iter()
        .map(|variant| {
            let tag = variant
                .pointer(&format!("/properties/{DISCRIMINATOR}/const"))
                .and_then(Value::as_str)
                .ok_or_else(|| EmitError(format!("{frame} variant without a const tag")))?;
            let def = ref_name(variant)
                .ok_or_else(|| EmitError(format!("{frame}.{tag} must wrap a named struct")))?;
            Ok((tag.to_string(), def.to_string()))
        })
        .collect()
}

/// Emit C# for `roots`: each a `(FrameName, schema)` pair whose schema is an
/// internally tagged `oneOf` over named structs.
pub fn emit(namespace: &str, roots: &[(&str, Value)]) -> Result<String, EmitError> {
    let mut defs: BTreeMap<String, Value> = BTreeMap::new();
    let mut bases: BTreeMap<String, String> = BTreeMap::new();
    let mut frames = Vec::new();

    for (frame, root) in roots {
        if let Some(root_defs) = root.get("$defs").and_then(Value::as_object) {
            for (name, def) in root_defs {
                match defs.get(name) {
                    Some(existing) if existing != def => {
                        return err(format!("conflicting definitions for {name}"))
                    }
                    _ => {
                        defs.insert(name.clone(), def.clone());
                    }
                }
            }
        }
        let variants = frame_variants(frame, root)?;
        for (_, def) in &variants {
            if let Some(prev) = bases.insert(def.clone(), (*frame).to_string()) {
                return err(format!("{def} is a variant of both {prev} and {frame}"));
            }
        }
        frames.push((*frame, variants));
    }

    let mut out = String::new();
    writeln!(out, "// <auto-generated>").ok();
    writeln!(out, "// Generated by: cargo run --bin generate_types").ok();
    writeln!(out, "// Source: src/endpoint/protocol.rs. Do not edit.").ok();
    writeln!(out, "// </auto-generated>").ok();
    writeln!(out, "#nullable enable\n").ok();
    writeln!(out, "using System.Collections.Generic;").ok();
    writeln!(out, "using System.Text.Json.Serialization;\n").ok();
    writeln!(out, "namespace {namespace};\n").ok();
    writeln!(out, "public static class ProtocolInfo\n{{").ok();
    writeln!(out, "{INDENT}public const uint Version = {PROTOCOL_VERSION};").ok();
    writeln!(out, "}}\n").ok();

    for (frame, variants) in &frames {
        writeln!(
            out,
            "[JsonPolymorphic(TypeDiscriminatorPropertyName = \"{DISCRIMINATOR}\", UnknownDerivedTypeHandling = JsonUnknownDerivedTypeHandling.FailSerialization)]"
        )
        .ok();
        for (tag, def) in variants {
            writeln!(out, "[JsonDerivedType(typeof({def}), \"{tag}\")]").ok();
        }
        writeln!(out, "public abstract record {frame};\n").ok();
    }

    for (name, def) in &defs {
        if let Some(variants) = def.get("enum").and_then(Value::as_array) {
            emit_enum(&mut out, name, variants)?;
        } else if def.get("type").and_then(Value::as_str) == Some("object") {
            emit_record(&mut out, name, def, bases.get(name).map(String::as_str))?;
        } else {
            return err(format!("unsupported definition {name}"));
        }
    }

    Ok(out.trim_end().to_string() + "\n")
}

pub fn endpoint_protocol_csharp() -> String {
    let schema = |s: schemars::Schema| serde_json::to_value(s).expect("schema serializes");
    emit(
        NAMESPACE,
        &[
            ("UplinkFrame", schema(schemars::schema_for!(UplinkFrame))),
            ("DownlinkFrame", schema(schemars::schema_for!(DownlinkFrame))),
        ],
    )
    .expect("endpoint protocol must stay within the C# emitter's supported shapes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn output_is_deterministic() {
        assert_eq!(endpoint_protocol_csharp(), endpoint_protocol_csharp());
    }

    #[test]
    fn every_frame_tag_is_a_derived_type() {
        let cs = endpoint_protocol_csharp();
        for tag in UplinkFrame::TAGS.iter().chain(DownlinkFrame::TAGS) {
            assert!(cs.contains(&format!("\"{tag}\")]")), "missing derived type for {tag}");
        }
        assert!(cs.contains("public abstract record UplinkFrame"));
        assert!(cs.contains("public abstract record DownlinkFrame"));
        assert!(cs.contains("TypeDiscriminatorPropertyName = \"type\""));
    }

    #[test]
    fn maps_field_shapes() {
        let cs = endpoint_protocol_csharp();
        assert!(cs.contains("public int? ExitCode { get; init; }"));
        assert!(cs.contains("public required ushort Status { get; init; }"));
        assert!(cs.contains("public required ulong Seq { get; init; }"));
        assert!(cs.contains("public IReadOnlyList<string> LogTail { get; init; } = [];"));
        assert!(cs.contains(
            "public IReadOnlyDictionary<string, string> Headers { get; init; } = new Dictionary<string, string>();"
        ));
        assert!(cs.contains("[JsonPropertyName(\"capabilities_hash\")]"));
        assert!(cs.contains("public required Capabilities Capabilities { get; init; }"));
        assert!(cs.contains("[JsonStringEnumMemberName(\"exited\")]"));
        assert!(cs.contains("public const uint Version = 1;"));
    }

    fn root(defs: Value, variants: &[(&str, &str)]) -> Value {
        let one_of: Vec<Value> = variants
            .iter()
            .map(|(tag, def)| {
                json!({"type":"object","properties":{"type":{"type":"string","const":tag}},
                       "$ref": format!("#/$defs/{def}"), "required":["type"]})
            })
            .collect();
        json!({"oneOf": one_of, "$defs": defs})
    }

    #[test]
    fn rejects_unsupported_number_shape() {
        let r = root(
            json!({"A":{"type":"object","properties":{"x":{"type":"number"}},"required":["x"]}}),
            &[("a", "A")],
        );
        assert!(emit(NAMESPACE, &[("Frame", r)]).is_err());
    }

    #[test]
    fn rejects_struct_shared_by_two_frame_enums() {
        let defs = json!({"A":{"type":"object","properties":{},"required":[]}});
        let r1 = root(defs.clone(), &[("a", "A")]);
        let r2 = root(defs, &[("a", "A")]);
        assert!(emit(NAMESPACE, &[("One", r1), ("Two", r2)]).is_err());
    }

    #[test]
    fn rejects_optional_scalar_without_null() {
        let r = root(
            json!({"A":{"type":"object","properties":{"x":{"type":"boolean","default":false}},"required":[]}}),
            &[("a", "A")],
        );
        assert!(emit(NAMESPACE, &[("Frame", r)]).is_err());
    }
}
