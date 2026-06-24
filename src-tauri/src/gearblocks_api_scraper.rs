use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

const DOCS_ROOT: &str = "https://www.gearblocksgame.com/apidoc/";
const HIERARCHY_PAGE: &str = "https://www.gearblocksgame.com/apidoc/hierarchy.html";
const NAMESPACES_PAGE: &str = "https://www.gearblocksgame.com/apidoc/namespaces.html";
const SOURCE: &str = "official-docs";
const SOURCE_VERSION_FALLBACK: &str = "doxygen";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearBlocksApiImportResult {
    pub source: String,
    pub source_version: String,
    pub docs_root: String,
    pub fetched_pages: usize,
    pub imported_type_count: usize,
    pub imported_member_count: usize,
    pub imported_parameter_count: usize,
    pub imported_enum_value_count: usize,
}

#[derive(Clone, Debug)]
pub struct GearBlocksApiScrape {
    pub source: String,
    pub source_version: String,
    pub docs_root: String,
    pub fetched_pages: usize,
    pub types: Vec<GearBlocksApiScrapedType>,
}

#[derive(Clone, Debug)]
pub struct GearBlocksApiScrapedType {
    pub namespace: String,
    pub type_name: String,
    pub type_kind: String,
    pub docs_url: String,
    pub notes: String,
    pub members: Vec<GearBlocksApiScrapedMember>,
    pub enum_values: Vec<GearBlocksApiScrapedEnumValue>,
}

#[derive(Clone, Debug)]
pub struct GearBlocksApiScrapedMember {
    pub member_key: String,
    pub member_name: String,
    pub signature: String,
    pub member_kind: String,
    pub return_type: String,
    pub is_readable: bool,
    pub is_writable: bool,
    pub is_invokable: bool,
    pub is_mutating: bool,
    pub docs_url: String,
    pub notes: String,
    pub parameters: Vec<GearBlocksApiScrapedParameter>,
}

#[derive(Clone, Debug)]
pub struct GearBlocksApiScrapedParameter {
    pub position: i64,
    pub parameter_name: String,
    pub parameter_type: String,
    pub default_value: String,
    pub is_optional: bool,
}

#[derive(Clone, Debug)]
pub struct GearBlocksApiScrapedEnumValue {
    pub position: i64,
    pub value_name: String,
    pub numeric_value: String,
    pub lua_name: String,
    pub description: String,
}

#[derive(Clone, Debug)]
struct TypeCandidate {
    namespace: String,
    type_name: String,
    type_kind: String,
    docs_url: String,
    notes: String,
}

pub fn scrape_official_gearblocks_api() -> Result<GearBlocksApiScrape, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("OverlayForge/gearblocks-api-index")
        .build()
        .map_err(|error| error.to_string())?;

    let mut fetched_pages = 0usize;
    let hierarchy = fetch_text(&client, HIERARCHY_PAGE)?;
    fetched_pages += 1;

    let mut candidates = parse_hierarchy_type_candidates(&hierarchy);
    let namespaces = fetch_text(&client, NAMESPACES_PAGE)?;
    fetched_pages += 1;
    let namespace_urls = parse_namespace_urls(&namespaces);
    let source_version =
        parse_source_version(&hierarchy).unwrap_or_else(|| SOURCE_VERSION_FALLBACK.to_string());

    let mut candidate_urls = candidates
        .iter()
        .map(|candidate| candidate.docs_url.clone())
        .collect::<BTreeSet<_>>();

    let mut enum_types = BTreeMap::<(String, String), GearBlocksApiScrapedType>::new();
    for namespace_url in namespace_urls {
        let Ok(namespace_html) = fetch_text(&client, &namespace_url) else {
            continue;
        };
        fetched_pages += 1;
        for candidate in parse_namespace_type_candidates(&namespace_html, &namespace_url) {
            if candidate_urls.insert(candidate.docs_url.clone()) {
                candidates.push(candidate);
            }
        }
        for enum_type in parse_namespace_enums(&namespace_html, &namespace_url) {
            enum_types.insert(
                (enum_type.namespace.clone(), enum_type.type_name.clone()),
                enum_type,
            );
        }
    }

    candidates.sort_by(|a, b| {
        a.namespace
            .cmp(&b.namespace)
            .then(a.type_name.cmp(&b.type_name))
    });

    let mut types = Vec::new();
    for candidate in candidates {
        let Ok(type_html) = fetch_text(&client, &candidate.docs_url) else {
            types.push(candidate.into_scraped_type(Vec::new()));
            continue;
        };
        fetched_pages += 1;
        let scraped = parse_type_page(candidate, &type_html);
        types.push(scraped);
    }

    for enum_type in enum_types.into_values() {
        if !types.iter().any(|api_type| {
            api_type.namespace == enum_type.namespace && api_type.type_name == enum_type.type_name
        }) {
            types.push(enum_type);
        }
    }

    Ok(GearBlocksApiScrape {
        source: SOURCE.to_string(),
        source_version,
        docs_root: DOCS_ROOT.to_string(),
        fetched_pages,
        types,
    })
}

impl TypeCandidate {
    fn into_scraped_type(
        self,
        members: Vec<GearBlocksApiScrapedMember>,
    ) -> GearBlocksApiScrapedType {
        GearBlocksApiScrapedType {
            namespace: self.namespace,
            type_name: self.type_name,
            type_kind: self.type_kind,
            docs_url: self.docs_url,
            notes: self.notes,
            members,
            enum_values: Vec::new(),
        }
    }
}

fn fetch_text(client: &reqwest::blocking::Client, url: &str) -> Result<String, String> {
    client
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("Failed to fetch {url}: {error}"))?
        .text()
        .map_err(|error| format!("Failed to read {url}: {error}"))
}

fn parse_hierarchy_type_candidates(html: &str) -> Vec<TypeCandidate> {
    let mut candidates = Vec::new();
    let mut seen_urls = BTreeSet::new();
    for row in html.split("<tr") {
        let Some(href) = first_href(row) else {
            continue;
        };
        let kind = docs_kind_from_href(&href);
        if kind.is_empty() {
            continue;
        }
        let Some(anchor_text) = first_anchor_text(row) else {
            continue;
        };
        let full_name = normalize_type_name(&strip_html(&anchor_text));
        if !full_name.contains('.') {
            continue;
        }
        let docs_url = absolutize_docs_url(&href);
        if !seen_urls.insert(docs_url.clone()) {
            continue;
        }
        let (namespace, type_name) = split_full_type_name(&full_name);
        candidates.push(TypeCandidate {
            namespace,
            type_name,
            type_kind: kind.to_string(),
            docs_url,
            notes: extract_desc_cell(row).unwrap_or_default(),
        });
    }
    candidates
}

fn parse_namespace_urls(html: &str) -> Vec<String> {
    let mut urls = BTreeSet::new();
    for fragment in html.split("href=\"").skip(1) {
        let href = fragment.split('"').next().unwrap_or_default();
        if href.starts_with("namespace_") && href.ends_with(".html") {
            urls.insert(absolutize_docs_url(href));
        }
    }
    urls.into_iter().collect()
}

fn parse_namespace_type_candidates(html: &str, _namespace_url: &str) -> Vec<TypeCandidate> {
    let namespace = parse_namespace_title(html).unwrap_or_default();
    if namespace.is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for row in html.split("<tr") {
        let Some(href) = first_href(row) else {
            continue;
        };
        let kind = docs_kind_from_href(&href);
        if kind.is_empty() {
            continue;
        }
        let Some(anchor_text) = first_anchor_text(row) else {
            continue;
        };
        let type_name = normalize_type_name(&strip_html(&anchor_text));
        if type_name.is_empty() || type_name.contains('.') {
            continue;
        }
        candidates.push(TypeCandidate {
            namespace: namespace.clone(),
            type_name,
            type_kind: kind.to_string(),
            docs_url: absolutize_docs_url(&href),
            notes: extract_desc_cell(row).unwrap_or_default(),
        });
    }

    candidates
}

fn parse_namespace_enums(html: &str, namespace_url: &str) -> Vec<GearBlocksApiScrapedType> {
    let namespace = parse_namespace_title(html).unwrap_or_default();
    if namespace.is_empty() {
        return Vec::new();
    }

    let mut enums = Vec::new();
    let mut section = String::new();
    for row in html.split("<tr") {
        if row.contains("groupheader") {
            section = strip_html(row).to_lowercase();
            continue;
        }
        if !section.contains("enumeration") && !section.contains("enum") {
            continue;
        }
        if !row.contains("memitem") {
            continue;
        }
        let right = extract_table_cell_by_class(row, "memItemRight").unwrap_or_default();
        let right_text = strip_html(&right);
        let Some(anchor_text) = first_anchor_text(&right) else {
            continue;
        };
        let type_name = normalize_type_name(&strip_html(&anchor_text));
        if type_name.is_empty() {
            continue;
        }
        let docs_url = first_href(&right)
            .map(|href| absolutize_docs_url(&href))
            .unwrap_or_else(|| namespace_url.to_string());
        let enum_values = parse_enum_values_from_row(&right_text);
        enums.push(GearBlocksApiScrapedType {
            namespace: namespace.clone(),
            type_name,
            type_kind: "enum".to_string(),
            docs_url,
            notes: extract_desc_cell(row).unwrap_or_default(),
            members: Vec::new(),
            enum_values,
        });
    }
    enums
}

fn parse_type_page(candidate: TypeCandidate, html: &str) -> GearBlocksApiScrapedType {
    let (namespace, type_name, type_kind) = parse_type_page_title(html).unwrap_or_else(|| {
        (
            candidate.namespace.clone(),
            candidate.type_name.clone(),
            candidate.type_kind.clone(),
        )
    });
    let notes = parse_type_summary(html).unwrap_or(candidate.notes);
    let members = parse_type_members(html, &candidate.docs_url, &type_name);

    GearBlocksApiScrapedType {
        namespace,
        type_name,
        type_kind,
        docs_url: candidate.docs_url,
        notes,
        members,
        enum_values: Vec::new(),
    }
}

fn parse_type_members(
    html: &str,
    type_docs_url: &str,
    type_name: &str,
) -> Vec<GearBlocksApiScrapedMember> {
    let mut members = Vec::new();
    let mut section = String::new();
    let rows: Vec<&str> = html.split("<tr").collect();
    for (index, row) in rows.iter().enumerate() {
        if row.contains("groupheader") {
            section = strip_html(row).to_lowercase();
            continue;
        }
        if !row.contains("memitem") {
            continue;
        }
        let member_kind = member_kind_from_section(&section);
        if member_kind.is_empty() {
            continue;
        }
        let left = extract_table_cell_by_class(row, "memItemLeft").unwrap_or_default();
        let right = extract_table_cell_by_class(row, "memItemRight").unwrap_or_default();
        let return_type = clean_member_type(&strip_html(&left));
        let right_text = strip_html(&right);
        let Some(anchor_text) = last_anchor_text(&right) else {
            continue;
        };
        let member_name = strip_html(&anchor_text);
        if member_name.is_empty() {
            continue;
        }
        let docs_url = first_href(&right)
            .map(|href| absolutize_fragment_url(type_docs_url, &href))
            .unwrap_or_else(|| type_docs_url.to_string());
        let description = rows
            .get(index + 1)
            .filter(|next_row| next_row.contains("memdesc"))
            .map(|next_row| strip_html(next_row))
            .unwrap_or_default();
        let signature = member_signature(&member_kind, &member_name, &right_text);
        let parameters = if member_kind == "method" {
            parse_parameters(&signature)
        } else {
            Vec::new()
        };
        members.push(GearBlocksApiScrapedMember {
            member_key: member_key(&member_name, &signature),
            member_name: member_name.clone(),
            signature,
            member_kind: member_kind.clone(),
            return_type,
            is_readable: member_kind == "property" && right_text.contains("[get]"),
            is_writable: member_kind == "property" && right_text.contains("[set]"),
            is_invokable: member_kind == "method",
            is_mutating: member_kind == "method" && is_mutating_member(type_name, &member_name),
            docs_url,
            notes: description,
            parameters,
        });
    }
    members
}

fn parse_enum_values_from_row(row_text: &str) -> Vec<GearBlocksApiScrapedEnumValue> {
    let Some(start) = row_text.find('{') else {
        return Vec::new();
    };
    let Some(end) = row_text[start + 1..].find('}') else {
        return Vec::new();
    };
    row_text[start + 1..start + 1 + end]
        .split(',')
        .enumerate()
        .filter_map(|(index, raw)| {
            let value_name = raw
                .split('=')
                .next()
                .unwrap_or_default()
                .trim()
                .trim_matches('|')
                .to_string();
            if value_name.is_empty() {
                return None;
            }
            let numeric_value = raw
                .split_once('=')
                .map(|(_, value)| value.trim().to_string())
                .unwrap_or_default();
            Some(GearBlocksApiScrapedEnumValue {
                position: index as i64,
                value_name,
                numeric_value,
                lua_name: String::new(),
                description: String::new(),
            })
        })
        .collect()
}

fn parse_parameters(signature: &str) -> Vec<GearBlocksApiScrapedParameter> {
    let Some(start) = signature.find('(') else {
        return Vec::new();
    };
    let Some(end) = signature.rfind(')') else {
        return Vec::new();
    };
    let params = signature[start + 1..end].trim();
    if params.is_empty() {
        return Vec::new();
    }

    split_parameters(params)
        .into_iter()
        .enumerate()
        .filter_map(|(position, raw)| {
            let (without_default, default_value) = raw
                .split_once('=')
                .map(|(left, right)| (left.trim(), right.trim().to_string()))
                .unwrap_or((raw.trim(), String::new()));
            let mut tokens = without_default.rsplitn(2, char::is_whitespace);
            let name = tokens.next().unwrap_or_default().trim();
            let parameter_type = tokens.next().unwrap_or_default().trim();
            if name.is_empty() {
                return None;
            }
            Some(GearBlocksApiScrapedParameter {
                position: position as i64,
                parameter_name: name.to_string(),
                parameter_type: parameter_type.to_string(),
                default_value,
                is_optional: raw.contains('='),
            })
        })
        .collect()
}

fn split_parameters(params: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (index, character) in params.char_indices() {
        match character {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(params[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(params[start..].trim());
    parts.into_iter().filter(|part| !part.is_empty()).collect()
}

fn parse_type_page_title(html: &str) -> Option<(String, String, String)> {
    let title = extract_div_title(html)?;
    let kind = if title.contains(" Interface Reference") {
        "interface"
    } else if title.contains(" Struct Reference") {
        "struct"
    } else if title.contains(" Class Reference") {
        "class"
    } else {
        return None;
    };
    let full_name = title
        .replace(" Interface Reference", "")
        .replace(" Struct Reference", "")
        .replace(" Class Reference", "");
    let (namespace, type_name) = split_full_type_name(&full_name);
    Some((namespace, type_name, kind.to_string()))
}

fn parse_namespace_title(html: &str) -> Option<String> {
    extract_div_title(html)
        .map(|title| title.replace(" Namespace Reference", ""))
        .map(|title| title.trim().to_string())
        .filter(|title| !title.is_empty())
}

fn parse_type_summary(html: &str) -> Option<String> {
    if let Some(textblock) = extract_between(html, "<div class=\"textblock\">", "</div>") {
        let summary = strip_html(&textblock);
        if !summary.is_empty() {
            return Some(summary);
        }
    }
    None
}

fn extract_div_title(html: &str) -> Option<String> {
    extract_between(html, "<div class=\"title\">", "</div>")
        .map(|title| strip_html(&title))
        .filter(|title| !title.is_empty())
}

fn parse_source_version(html: &str) -> Option<String> {
    let generated = html
        .split("Generated by")
        .nth(1)
        .and_then(|fragment| fragment.split("</small>").next())?;
    let text = strip_html(generated);
    if text.is_empty() {
        return None;
    }
    Some(text)
}

fn member_kind_from_section(section: &str) -> String {
    if section.contains("member function") || section.contains("methods") {
        "method".to_string()
    } else if section.contains("properties") {
        "property".to_string()
    } else if section.contains("attribute") || section.contains("data fields") {
        "field".to_string()
    } else {
        String::new()
    }
}

fn member_signature(member_kind: &str, member_name: &str, right_text: &str) -> String {
    if member_kind != "method" {
        return member_name.to_string();
    }
    if let Some(start) = right_text.find('(') {
        let suffix = right_text[start..].trim().replace(" )", ")");
        format!("{member_name}{suffix}")
    } else {
        format!("{member_name}()")
    }
}

fn member_key(member_name: &str, signature: &str) -> String {
    if let Some(start) = signature.find('(') {
        format!("{member_name}{}", &signature[start..])
    } else {
        member_name.to_string()
    }
}

fn is_mutating_member(_type_name: &str, member_name: &str) -> bool {
    let lower = member_name.to_lowercase();
    [
        "add",
        "assign",
        "charge",
        "clear",
        "create",
        "delete",
        "destroy",
        "discharge",
        "duplicate",
        "freeze",
        "invoke",
        "move",
        "replace",
        "set",
        "spawn",
        "sync",
        "toggle",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

fn docs_kind_from_href(href: &str) -> &'static str {
    if href.starts_with("class_") {
        "class"
    } else if href.starts_with("interface_") {
        "interface"
    } else if href.starts_with("struct_") {
        "struct"
    } else {
        ""
    }
}

fn first_href(html: &str) -> Option<String> {
    html.split("href=\"")
        .nth(1)
        .and_then(|fragment| fragment.split('"').next())
        .map(|href| href.to_string())
}

fn first_anchor_text(html: &str) -> Option<String> {
    let anchor_start = html.find("<a ")?;
    let text_start = html[anchor_start..].find('>')? + anchor_start + 1;
    let text_end = html[text_start..].find("</a>")? + text_start;
    Some(html[text_start..text_end].to_string())
}

fn last_anchor_text(html: &str) -> Option<String> {
    let anchor_start = html.rfind("<a ")?;
    let text_start = html[anchor_start..].find('>')? + anchor_start + 1;
    let text_end = html[text_start..].find("</a>")? + text_start;
    Some(html[text_start..text_end].to_string())
}

fn extract_desc_cell(row: &str) -> Option<String> {
    extract_table_cell_by_class(row, "desc")
        .map(|cell| strip_html(&cell))
        .filter(|cell| !cell.is_empty())
}

fn extract_table_cell_by_class(row: &str, class_name: &str) -> Option<String> {
    let class_index = row.find(class_name)?;
    let start = row[class_index..].find('>')? + class_index + 1;
    let end = row[start..].find("</td>")? + start;
    Some(row[start..end].to_string())
}

fn extract_between(value: &str, start_marker: &str, end_marker: &str) -> Option<String> {
    let start = value.find(start_marker)? + start_marker.len();
    let end = value[start..].find(end_marker)? + start;
    Some(value[start..end].to_string())
}

fn split_full_type_name(full_name: &str) -> (String, String) {
    let normalized = normalize_type_name(full_name);
    let Some(index) = normalized.rfind('.') else {
        return (String::new(), normalized);
    };
    (
        normalized[..index].trim().to_string(),
        normalized[index + 1..].trim().to_string(),
    )
}

fn normalize_type_name(value: &str) -> String {
    value
        .replace(" &lt; ", "<")
        .replace(" &gt;", ">")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("< ", "<")
        .replace(" >", ">")
}

fn clean_member_type(value: &str) -> String {
    value
        .replace('\u{a0}', " ")
        .replace(" static", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn absolutize_docs_url(href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else {
        format!("{DOCS_ROOT}{}", href.trim_start_matches("./"))
    }
}

fn absolutize_fragment_url(base_url: &str, href: &str) -> String {
    if href.starts_with('#') {
        format!("{base_url}{href}")
    } else {
        absolutize_docs_url(href)
    }
}

fn strip_html(value: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                output.push(' ');
            }
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    decode_entities(&output)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn decode_entities(value: &str) -> String {
    value
        .replace("&#160;", " ")
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#9670;", " ")
        .replace("&#9658;", " ")
        .replace("&#9660;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_method_parameters_with_defaults() {
        let parameters = parse_parameters(
            "CreateAttachment(AttachmentTypeFlags type, IPart ownerPart, bool invokeAsPlayer=false)",
        );

        assert_eq!(parameters.len(), 3);
        assert_eq!(parameters[0].parameter_name, "type");
        assert_eq!(parameters[0].parameter_type, "AttachmentTypeFlags");
        assert_eq!(parameters[2].parameter_name, "invokeAsPlayer");
        assert_eq!(parameters[2].default_value, "false");
        assert!(parameters[2].is_optional);
    }

    #[test]
    fn parses_type_page_title() {
        let html = r#"<div class="title">SmashHammer.GearBlocks.Construction.IPart Interface Reference</div>"#;
        let (namespace, type_name, kind) = parse_type_page_title(html).unwrap();

        assert_eq!(namespace, "SmashHammer.GearBlocks.Construction");
        assert_eq!(type_name, "IPart");
        assert_eq!(kind, "interface");
    }
}
