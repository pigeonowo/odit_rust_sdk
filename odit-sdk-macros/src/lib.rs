//! Zero-dependency procedural macros for the ODIT Rust Plugin SDK (`odit-sdk`).
//!
//! Provides `#[odit_plugin(...)]` to declare a plugin struct's manifest metadata
//! and automatically export all required ODIT Host ABI v1.0.0 WebAssembly entrypoints.

use proc_macro::{TokenStream, TokenTree};

fn parse_string_lit(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn find_struct_name(item: &TokenStream) -> Option<String> {
    let mut iter = item.clone().into_iter();
    while let Some(tt) = iter.next() {
        if let TokenTree::Ident(id) = tt {
            if id.to_string() == "struct" {
                if let Some(TokenTree::Ident(name)) = iter.next() {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

/// Marks a struct as an ODIT WASM plugin, generating its JSON manifest and
/// exporting all required ABI v1.0.0 WebAssembly functions automatically.
///
/// # Example
/// ```ignore
/// use odit_sdk::prelude::*;
///
/// #[odit_plugin(
///     id = "user.my_plugin",
///     title = "My Rust Plugin",
///     version = "0.1.0",
///     slot = RightGrid,
///     width = 360.0,
///     height = 240.0,
///     event_mask = 0xFFFFFFFF,
///     allow_multiple = false,
/// )]
/// #[derive(Default)]
/// pub struct MyPlugin {
///     clicks: u32,
/// }
/// ```
#[proc_macro_attribute]
pub fn odit_plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut id = String::new();
    let mut title = String::new();
    let mut version = String::from("0.1.0");
    let mut slot: u32 = 0; // RightGrid
    let mut width: u32 = 360;
    let mut height: u32 = 240;
    let mut event_mask: String = String::from("0xFFFFFFFFu32");
    let mut allow_multiple = false;

    // Parse comma-separated `key = value` pairs from `attr`
    let tokens: Vec<TokenTree> = attr.into_iter().collect();
    let mut i = 0;
    while i < tokens.len() {
        let key = match &tokens[i] {
            TokenTree::Ident(ident) => ident.to_string(),
            TokenTree::Punct(p) if p.as_char() == ',' => {
                i += 1;
                continue;
            }
            other => {
                return format!(
                    "compile_error!(\"#[odit_plugin]: expected argument name, found `{}`\");",
                    other
                )
                .parse()
                .unwrap();
            }
        };
        i += 1;

        // Expect `=`
        match tokens.get(i) {
            Some(TokenTree::Punct(p)) if p.as_char() == '=' => i += 1,
            _ => {
                return format!(
                    "compile_error!(\"#[odit_plugin]: expected `=` after `{}`\");",
                    key
                )
                .parse()
                .unwrap();
            }
        }

        // Collect value tokens until `,` or end
        let mut val_parts = Vec::new();
        while i < tokens.len() {
            if let TokenTree::Punct(p) = &tokens[i] {
                if p.as_char() == ',' {
                    i += 1;
                    break;
                }
            }
            val_parts.push(tokens[i].to_string());
            i += 1;
        }
        let val = val_parts.join("");

        match key.as_str() {
            "id" => id = parse_string_lit(&val),
            "title" => title = parse_string_lit(&val),
            "version" => version = parse_string_lit(&val),
            "slot" | "default_slot" => {
                slot = match val.as_str() {
                    "RightGrid" | "DockSlot::RightGrid" | "0" => 0,
                    "LeftSide" | "DockSlot::LeftSide" | "1" => 1,
                    "Bottom" | "DockSlot::Bottom" | "2" => 2,
                    "Floating" | "DockSlot::Floating" | "3" => 3,
                    "MainArea" | "DockSlot::MainArea" | "4" => 4,
                    other => other.parse::<u32>().unwrap_or(0),
                };
            }
            "width" | "default_width" => {
                width = val.parse::<f32>().map(|v| v.max(64.0) as u32).unwrap_or(360);
            }
            "height" | "default_height" => {
                height = val.parse::<f32>().map(|v| v.max(64.0) as u32).unwrap_or(240);
            }
            "event_mask" => {
                event_mask = val;
            }
            "allow_multiple" => {
                allow_multiple = val == "true";
            }
            unknown => {
                return format!(
                    "compile_error!(\"#[odit_plugin]: unknown attribute `{}`\");",
                    unknown
                )
                .parse()
                .unwrap();
            }
        }
    }

    let struct_name = match find_struct_name(&item) {
        Some(n) => n,
        None => {
            return "compile_error!(\"#[odit_plugin] must be applied to a struct definition\");"
                .parse()
                .unwrap();
        }
    };

    if id.is_empty() {
        id = format!("user.{}", struct_name.to_lowercase());
    }
    if title.is_empty() {
        title = struct_name.clone();
    }

    // Parse event_mask if it's a numeric literal; otherwise default to 0xFFFFFFFF.
    let mask_val: u32 = if let Some(hex) = event_mask
        .trim_end_matches("u32")
        .strip_prefix("0x")
        .or_else(|| event_mask.trim_end_matches("u32").strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16).unwrap_or(0xFFFF_FFFF)
    } else {
        event_mask
            .trim_end_matches("u32")
            .parse::<u32>()
            .unwrap_or(0xFFFF_FFFF)
    };

    let manifest_json = format!(
        "{{\"id\":\"{}\",\"title\":\"{}\",\"version\":\"{}\",\"default_slot\":{},\"default_width\":{},\"default_height\":{},\"event_mask\":{},\"allow_multiple\":{}}}",
        json_escape(&id),
        json_escape(&title),
        json_escape(&version),
        slot,
        width,
        height,
        mask_val,
        if allow_multiple { "true" } else { "false" }
    );

    let generated = format!(
        r#"
impl ::odit_sdk::PluginMeta for {struct_name} {{
    const MANIFEST_JSON: &'static str = {manifest_json:?};
}}

::odit_sdk::__export_plugin!({struct_name});
"#,
        struct_name = struct_name,
        manifest_json = manifest_json,
    );

    let mut out = item;
    out.extend(generated.parse::<TokenStream>().unwrap());
    out
}
