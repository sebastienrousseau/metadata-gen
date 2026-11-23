// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Schema.org structured data as JSON-LD.
//!
//! [`json_ld`](crate::schema_org::json_ld) maps flat front matter onto an `Article`, `Product` or
//! `Organization` object, and [`breadcrumb_list`](crate::schema_org::breadcrumb_list) builds a
//! `BreadcrumbList` from a trail of pages. Both return the JSON text;
//! [`json_ld_script`](crate::schema_org::json_ld_script) wraps it in the `<script>` element a page embeds.
//! The JSON is written by hand, so the module needs no JSON dependency
//! and works in a `no_std + alloc` build. Keys that are absent from the
//! metadata are left out; nothing is invented.
//!
//! ```
//! use metadata_gen::schema_org::{json_ld, json_ld_script, SchemaType};
//! use metadata_gen::MetadataMap;
//!
//! let mut meta = MetadataMap::new();
//! meta.insert("title".into(), "Hello".into());
//! meta.insert("author".into(), "Ada".into());
//! meta.insert("date".into(), "2026-01-02".into());
//! let json = json_ld(SchemaType::Article, &meta);
//! assert_eq!(
//!     json,
//!     r#"{"@context":"https://schema.org","@type":"Article","headline":"Hello","datePublished":"2026-01-02","author":{"@type":"Person","name":"Ada"}}"#
//! );
//! assert!(json_ld_script(&json).starts_with("<script type=\"application/ld+json\">"));
//! ```

use crate::MetadataMap;
use alloc::string::String;
use alloc::vec::Vec;

/// The Schema.org type [`json_ld`] produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SchemaType {
    /// <https://schema.org/Article>: `title` is the headline.
    Article,
    /// <https://schema.org/Product>, with an `Offer` when `price` is set.
    Product,
    /// <https://schema.org/Organization>.
    Organization,
}

/// A JSON value as this module writes it.
enum Json {
    Str(String),
    Int(usize),
    Obj(Vec<(&'static str, Json)>),
    Arr(Vec<Json>),
}

/// Metadata keys read for each property, first match wins.
type Mapping = &'static [(&'static str, &'static [&'static str])];

const ARTICLE: Mapping = &[
    ("headline", &["title", "headline"]),
    ("description", &["description"]),
    ("image", &["image", "og:image"]),
    ("datePublished", &["date", "datePublished"]),
    (
        "dateModified",
        &["last_modified", "updated", "dateModified"],
    ),
    ("url", &["permalink", "url"]),
    ("keywords", &["keywords"]),
];

const PRODUCT: Mapping = &[
    ("name", &["name", "title"]),
    ("description", &["description"]),
    ("image", &["image", "og:image"]),
    ("sku", &["sku"]),
    ("url", &["permalink", "url"]),
];

const ORGANIZATION: Mapping = &[
    ("name", &["name", "title"]),
    ("description", &["description"]),
    ("url", &["url", "permalink"]),
    ("logo", &["logo", "image"]),
    ("email", &["email"]),
];

/// Builds the JSON-LD object for `kind` from `metadata`.
///
/// Besides the flat properties, an `Article` gets `author` as a `Person`
/// (from `author`) and `publisher` as an `Organization` (from
/// `publisher`); a `Product` gets `brand` as a `Brand` (from `brand`) and
/// `offers` as an `Offer` (from `price`, `currency`, `availability`).
pub fn json_ld(kind: SchemaType, metadata: &MetadataMap) -> String {
    let (name, mapping) = match kind {
        SchemaType::Article => ("Article", ARTICLE),
        SchemaType::Product => ("Product", PRODUCT),
        SchemaType::Organization => ("Organization", ORGANIZATION),
    };
    let mut object = header(name);
    for (property, keys) in mapping {
        if let Some(value) = first(metadata, keys) {
            object.push((property, Json::Str(value)));
        }
    }
    match kind {
        SchemaType::Article => article_extras(metadata, &mut object),
        SchemaType::Product => product_extras(metadata, &mut object),
        SchemaType::Organization => {}
    }
    render(&Json::Obj(object))
}

/// Builds a `BreadcrumbList` from `(name, url)` pairs, in trail order.
///
/// # Example
///
/// ```
/// use metadata_gen::schema_org::breadcrumb_list;
///
/// let json = breadcrumb_list(&[("Home", "https://example.com/")]);
/// assert!(json.contains(r#""itemListElement":[{"@type":"ListItem","position":1,"name":"Home","item":"https://example.com/"}]"#));
/// ```
pub fn breadcrumb_list(trail: &[(&str, &str)]) -> String {
    let items = trail
        .iter()
        .enumerate()
        .map(|(i, (name, url))| {
            Json::Obj(alloc::vec![
                ("@type", text("ListItem")),
                ("position", Json::Int(i + 1)),
                ("name", text(name)),
                ("item", text(url)),
            ])
        })
        .collect();
    let mut object = header("BreadcrumbList");
    object.push(("itemListElement", Json::Arr(items)));
    render(&Json::Obj(object))
}

/// Wraps JSON-LD text in `<script type="application/ld+json">`.
///
/// The JSON this module writes escapes `<`, so it cannot close the
/// element early; pass only JSON from [`json_ld`] or
/// [`breadcrumb_list`], or JSON escaped the same way.
pub fn json_ld_script(json: &str) -> String {
    alloc::format!(
        "<script type=\"application/ld+json\">{json}</script>"
    )
}

fn header(kind: &str) -> Vec<(&'static str, Json)> {
    alloc::vec![
        ("@context", text("https://schema.org")),
        ("@type", text(kind)),
    ]
}

fn text(s: &str) -> Json {
    Json::Str(String::from(s))
}

fn first(metadata: &MetadataMap, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|k| metadata.get(*k))
        .filter(|v| !v.is_empty())
        .cloned()
}

/// `{"@type": kind, "name": value}` when `key` is present.
fn typed_name(
    metadata: &MetadataMap,
    key: &str,
    kind: &str,
) -> Option<Json> {
    let name = first(metadata, &[key])?;
    Some(Json::Obj(alloc::vec![
        ("@type", text(kind)),
        ("name", Json::Str(name)),
    ]))
}

fn article_extras(
    metadata: &MetadataMap,
    object: &mut Vec<(&'static str, Json)>,
) {
    if let Some(author) = typed_name(metadata, "author", "Person") {
        object.push(("author", author));
    }
    if let Some(publisher) =
        typed_name(metadata, "publisher", "Organization")
    {
        object.push(("publisher", publisher));
    }
}

fn product_extras(
    metadata: &MetadataMap,
    object: &mut Vec<(&'static str, Json)>,
) {
    if let Some(brand) = typed_name(metadata, "brand", "Brand") {
        object.push(("brand", brand));
    }
    let Some(price) = first(metadata, &["price"]) else {
        return;
    };
    let mut offer = alloc::vec![
        ("@type", text("Offer")),
        ("price", Json::Str(price))
    ];
    if let Some(currency) =
        first(metadata, &["currency", "priceCurrency"])
    {
        offer.push(("priceCurrency", Json::Str(currency)));
    }
    if let Some(availability) = first(metadata, &["availability"]) {
        offer.push(("availability", Json::Str(availability)));
    }
    object.push(("offers", Json::Obj(offer)));
}

fn render(value: &Json) -> String {
    let mut out = String::new();
    write(value, &mut out);
    out
}

fn write(value: &Json, out: &mut String) {
    match value {
        Json::Str(s) => write_str(s, out),
        Json::Int(n) => out.push_str(&alloc::format!("{n}")),
        Json::Arr(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write(item, out);
            }
            out.push(']');
        }
        Json::Obj(entries) => {
            out.push('{');
            for (i, (key, item)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_str(key, out);
                out.push(':');
                write(item, out);
            }
            out.push('}');
        }
    }
}

/// A JSON string literal. `<`, `>` and `&` are escaped as well, so the
/// text is safe inside an HTML `<script>` element.
fn write_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<'
            | '>'
            | '&'
            | '\u{0}'..='\u{1f}'
            | '\u{2028}'
            | '\u{2029}' => {
                out.push_str(&alloc::format!("\\u{:04x}", c as u32));
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    fn meta(pairs: &[(&str, &str)]) -> MetadataMap {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn product_with_offer_and_brand() {
        let json = json_ld(
            SchemaType::Product,
            &meta(&[
                ("title", "Widget"),
                ("brand", "Acme"),
                ("price", "9.99"),
                ("currency", "EUR"),
                ("availability", "https://schema.org/InStock"),
                ("sku", "W-1"),
            ]),
        );
        assert_eq!(
            json,
            r#"{"@context":"https://schema.org","@type":"Product","name":"Widget","sku":"W-1","brand":{"@type":"Brand","name":"Acme"},"offers":{"@type":"Offer","price":"9.99","priceCurrency":"EUR","availability":"https://schema.org/InStock"}}"#
        );
    }

    #[test]
    fn product_without_price_has_no_offer() {
        let json =
            json_ld(SchemaType::Product, &meta(&[("name", "W")]));
        assert!(!json.contains("offers"));
        assert!(!json.contains("brand"));
    }

    #[test]
    fn organization_and_article_publisher() {
        let org = json_ld(
            SchemaType::Organization,
            &meta(&[("name", "Acme"), ("logo", "https://e.com/l.png")]),
        );
        assert!(org.contains(r#""@type":"Organization","name":"Acme","logo":"https://e.com/l.png""#));
        let article = json_ld(
            SchemaType::Article,
            &meta(&[
                ("title", "T"),
                ("publisher", "Acme"),
                ("description", ""),
            ]),
        );
        assert!(article.contains(
            r#""publisher":{"@type":"Organization","name":"Acme"}"#
        ));
        assert!(
            !article.contains("description"),
            "empty values are left out"
        );
    }

    #[test]
    fn strings_are_escaped_for_json_and_html() {
        let json = json_ld(
            SchemaType::Article,
            &meta(&[("title", "a\"b\\c\n\r\t</script>&\u{1}\u{2028}")]),
        );
        assert!(json.contains(
            r#""headline":"a\"b\\c\n\r\t\u003c/script\u003e\u0026\u0001\u2028""#
        ));
    }

    #[test]
    fn article_author_and_script_wrapper() {
        let json = json_ld(
            SchemaType::Article,
            &meta(&[("title", "T"), ("author", "Ada")]),
        );
        assert!(json
            .contains(r#""author":{"@type":"Person","name":"Ada"}"#));
        assert_eq!(
            json_ld_script("{}"),
            "<script type=\"application/ld+json\">{}</script>"
        );
    }

    #[test]
    fn breadcrumbs_are_numbered_from_one() {
        let json =
            breadcrumb_list(&[("Home", "/"), ("Blog", "/blog/")]);
        assert!(json
            .contains(r#""position":2,"name":"Blog","item":"/blog/""#));
        assert_eq!(breadcrumb_list(&[]).matches("[]").count(), 1);
    }
}
