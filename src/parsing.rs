use std::collections::BTreeMap;

use crate::models::{EventDetail, Field, Row, SearchResult};

const BASE_URL: &str = "https://vvz.ruhr-uni-bochum.de";

pub fn discover_term_guid(html: &str, final_url: &str) -> Option<String> {
    query_param(final_url, "tguid").or_else(|| {
        let lower = html.to_lowercase();
        lower.find("tguid=").map(|index| {
            let start = index + "tguid=".len();
            let value = html[start..]
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == 'x' || *ch == 'X')
                .collect::<String>();
            html_decode(&value)
        })
    })
}

pub fn parse_search_results(html: &str, base_url: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();

    for row in tag_fragments(html, "tr") {
        if !row.to_lowercase().contains("event.asp") {
            continue;
        }
        let Some(href) = first_event_href(&row) else {
            continue;
        };
        let url = absolutize(base_url, &href);
        if results
            .iter()
            .any(|result: &SearchResult| result.url == url)
        {
            continue;
        }
        let title = cell_by_id(&row, "EVENTLIST_TITLE")
            .and_then(|cell| first_anchor_text(&cell))
            .or_else(|| first_anchor_text(&row))
            .unwrap_or_default();
        if title.is_empty() {
            continue;
        }
        let summary = eventlist_summary(&row);
        results.push(SearchResult {
            event_guid: query_param(&url, "gguid"),
            term_guid: query_param(&url, "tguid"),
            title,
            url,
            summary,
        });
    }

    for anchor in anchor_fragments(html) {
        let Some(href) = attr(&anchor.open_tag, "href") else {
            continue;
        };
        if !href.to_lowercase().contains("event.asp") {
            continue;
        }
        let url = absolutize(base_url, &href);
        if results.iter().any(|result| result.url == url) {
            continue;
        }
        let title = clean_text(&strip_tags(&anchor.inner_html));
        if title.is_empty() {
            continue;
        }
        results.push(SearchResult {
            event_guid: query_param(&url, "gguid"),
            term_guid: query_param(&url, "tguid"),
            title,
            url,
            summary: String::new(),
        });
    }

    results
}

pub fn parse_fields(html: &str, base_url: &str) -> Vec<Field> {
    let mut fields = Vec::new();
    for anchor in anchor_fragments(html) {
        let Some(href) = attr(&anchor.open_tag, "href") else {
            continue;
        };
        let url = absolutize(base_url, &href);
        let value = query_param(&url, "field");
        let guid = query_param(&url, "gguid");
        if value.is_none() && guid.is_none() {
            continue;
        }
        let name = clean_text(&strip_tags(&anchor.inner_html));
        if name.is_empty() {
            continue;
        }
        let term_guid = query_param(&url, "tguid");
        fields.push(Field {
            name,
            url,
            value,
            guid,
            term_guid,
        });
    }
    fields
}

pub fn parse_event_detail(html: &str, url: &str) -> EventDetail {
    let fields = parse_key_value_fields(html);
    let title = fields
        .get("Titel")
        .cloned()
        .or_else(|| first_heading(html))
        .or_else(|| first_title(html))
        .unwrap_or_else(|| "Course".to_string());

    EventDetail {
        title,
        url: url.to_string(),
        event_guid: query_param(url, "gguid"),
        term_guid: query_param(url, "tguid"),
        fields,
        sections: parse_sections(html),
        description: parse_description(html),
    }
}

pub fn event_url(event_guid: &str, term_guid: &str, lang: &str) -> String {
    format!(
        "{BASE_URL}/campus/all/event.asp?objgguid=NEW&from=vvz&gguid={}&mode=own&tguid={}&lang={}",
        url_encode(event_guid),
        url_encode(term_guid),
        url_encode(lang)
    )
}

pub fn url_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else if byte == b' ' {
            encoded.push('+');
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn parse_key_value_fields(html: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    for table in tag_fragments(html, "table") {
        if has_class(&table, "listview") || has_class(&table, "collapsiblelist") {
            continue;
        }
        for row in tag_fragments(&table, "tr") {
            let cells = cell_fragments(&row);
            if cells.len() != 2 {
                continue;
            }
            let key = cell_label(&cells[0]).trim_end_matches(':').to_string();
            let value = cell_value(&cells[1]);
            if key.is_empty() || value.is_empty() || key.chars().count() > 60 {
                continue;
            }
            fields.insert(key, value);
        }
    }
    fields
}

fn parse_sections(html: &str) -> BTreeMap<String, Vec<Row>> {
    let mut sections = BTreeMap::new();
    let mut cursor = 0;
    while let Some((heading_start, heading_end, heading_text)) = next_heading(html, cursor) {
        let next_start = next_heading(html, heading_end)
            .map(|(start, _, _)| start)
            .unwrap_or(html.len());
        let block = &html[heading_end..next_start];
        for table in tag_fragments(block, "table") {
            if has_class(&table, "displayfields") {
                continue;
            }
            let rows = table_as_records(&table);
            if !rows.is_empty() {
                sections.insert(heading_text.clone(), rows);
                break;
            }
        }
        cursor = heading_start + 1;
    }
    sections
}

fn parse_description(html: &str) -> String {
    let lower = html.to_lowercase();
    for needle in ["beschreibung", "inhalt", "kommentar"] {
        if let Some(index) = lower.find(needle) {
            let block = &html[index..html.len().min(index + 2500)];
            let text = clean_text(&strip_tags(block));
            if !text.is_empty() {
                return text;
            }
        }
    }
    String::new()
}

fn table_as_records(table: &str) -> Vec<Row> {
    let rows = tag_fragments(table, "tr");
    if rows.is_empty() {
        return Vec::new();
    }

    let first_cells = cell_fragments(&rows[0]);
    let has_header = first_cells.iter().any(|cell| starts_with_tag(cell, "th"));
    let headers: Vec<String> = first_cells
        .iter()
        .map(|cell| clean_text(&strip_tags(cell)))
        .collect();
    let data_rows = if has_header { &rows[1..] } else { &rows[..] };
    let mut records = Vec::new();

    for row in data_rows {
        let values: Vec<String> = cell_fragments(row)
            .iter()
            .map(|cell| clean_text(&strip_tags(cell)))
            .filter(|value| !value.is_empty())
            .collect();
        if values.is_empty() {
            continue;
        }
        let mut record = Row::new();
        if has_header && values.len() == headers.len() {
            for (header, value) in headers.iter().zip(values.iter()) {
                if !header.is_empty() && !value.is_empty() {
                    record.insert(header.clone(), value.clone());
                }
            }
        } else if values.len() == 1 {
            record.insert("value".to_string(), values[0].clone());
        } else {
            for (index, value) in values.iter().enumerate() {
                record.insert(format!("column_{}", index + 1), value.clone());
            }
        }
        if !record.is_empty() {
            records.push(record);
        }
    }
    records
}

fn eventlist_summary(row: &str) -> String {
    [
        cell_by_id(row, "EVENTLIST_COURSENUMBER"),
        cell_by_id(row, "EVENTLIST_LECTURER"),
        cell_by_id(row, "EVENTLIST_SWS").or_else(|| cell_by_id(row, "EVENTLIST_TYPE")),
    ]
    .into_iter()
    .flatten()
    .map(|cell| clean_text(&strip_tags(&cell)))
    .filter(|text| !text.is_empty())
    .collect::<Vec<_>>()
    .join(" | ")
}

fn first_event_href(html: &str) -> Option<String> {
    anchor_fragments(html).into_iter().find_map(|anchor| {
        attr(&anchor.open_tag, "href").filter(|href| href.to_lowercase().contains("event.asp"))
    })
}

fn first_anchor_text(html: &str) -> Option<String> {
    anchor_fragments(html)
        .into_iter()
        .map(|anchor| clean_text(&strip_tags(&anchor.inner_html)))
        .find(|text| !text.is_empty())
}

fn cell_by_id(row: &str, needle: &str) -> Option<String> {
    let needle = needle.to_uppercase();
    cell_fragments(row).into_iter().find(|cell| {
        let open = open_tag(cell);
        open.to_uppercase().contains(&needle)
    })
}

fn cell_label(cell: &str) -> String {
    first_fragment_with_class(cell, "label-wrap")
        .map(|fragment| clean_text(&strip_tags(&fragment)))
        .unwrap_or_else(|| clean_text(&strip_tags(cell)))
}

fn cell_value(cell: &str) -> String {
    first_fragment_with_class(cell, "cas-extended-field-value")
        .map(|fragment| clean_text(&strip_tags(&fragment)))
        .unwrap_or_else(|| clean_text(&strip_tags(cell)))
}

fn first_heading(html: &str) -> Option<String> {
    next_heading(html, 0).map(|(_, _, text)| text)
}

fn first_title(html: &str) -> Option<String> {
    tag_fragments(html, "title")
        .into_iter()
        .map(|fragment| clean_text(&strip_tags(&fragment)))
        .find(|text| !text.is_empty())
}

fn next_heading(html: &str, from: usize) -> Option<(usize, usize, String)> {
    let lower = html.to_lowercase();
    let h2 = lower[from..].find("<h2").map(|offset| from + offset);
    let h3 = lower[from..].find("<h3").map(|offset| from + offset);
    let start = match (h2, h3) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => return None,
    };
    let tag = &lower[start + 1..start + 3];
    let end_tag = format!("</{tag}>");
    let end = lower[start..]
        .find(&end_tag)
        .map(|offset| start + offset + end_tag.len())?;
    let text = clean_text(&strip_tags(&html[start..end]))
        .trim_start_matches(|ch: char| !ch.is_alphanumeric())
        .to_string();
    Some((start, end, text))
}

fn tag_fragments(html: &str, tag: &str) -> Vec<String> {
    let lower = html.to_lowercase();
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut fragments = Vec::new();
    let mut cursor = 0;

    while let Some(relative_start) = lower[cursor..].find(&open) {
        let start = cursor + relative_start;
        let Some(relative_close) = lower[start..].find(&close) else {
            break;
        };
        let end = start + relative_close + close.len();
        fragments.push(html[start..end].to_string());
        cursor = end;
    }

    fragments
}

fn cell_fragments(row: &str) -> Vec<String> {
    let lower = row.to_lowercase();
    let mut cells = Vec::new();
    let mut cursor = 0;

    while cursor < row.len() {
        let td = lower[cursor..].find("<td").map(|offset| cursor + offset);
        let th = lower[cursor..].find("<th").map(|offset| cursor + offset);
        let Some(start) = choose_min(td, th) else {
            break;
        };
        let tag = &lower[start + 1..start + 3];
        let close = format!("</{tag}>");
        let Some(relative_end) = lower[start..].find(&close) else {
            break;
        };
        let end = start + relative_end + close.len();
        cells.push(row[start..end].to_string());
        cursor = end;
    }

    cells
}

#[derive(Debug)]
struct Anchor {
    open_tag: String,
    inner_html: String,
}

fn anchor_fragments(html: &str) -> Vec<Anchor> {
    let lower = html.to_lowercase();
    let mut anchors = Vec::new();
    let mut cursor = 0;

    while let Some(relative_start) = lower[cursor..].find("<a") {
        let start = cursor + relative_start;
        let Some(relative_open_end) = lower[start..].find('>') else {
            break;
        };
        let open_end = start + relative_open_end + 1;
        let Some(relative_close) = lower[open_end..].find("</a>") else {
            break;
        };
        let close_start = open_end + relative_close;
        anchors.push(Anchor {
            open_tag: html[start..open_end].to_string(),
            inner_html: html[open_end..close_start].to_string(),
        });
        cursor = close_start + "</a>".len();
    }

    anchors
}

fn first_fragment_with_class(html: &str, class_name: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let mut cursor = 0;

    while let Some(relative_start) = lower[cursor..].find('<') {
        let start = cursor + relative_start;
        if lower[start..].starts_with("</") {
            cursor = start + 2;
            continue;
        }

        let Some(relative_open_end) = lower[start..].find('>') else {
            break;
        };
        let open_end = start + relative_open_end + 1;
        let open = &html[start..open_end];
        if !has_class(open, class_name) {
            cursor = open_end;
            continue;
        }

        let tag = tag_name(open)?;
        let close = format!("</{}>", tag.to_lowercase());
        let end = lower[open_end..]
            .find(&close)
            .map(|offset| open_end + offset + close.len())
            .unwrap_or(open_end);
        return Some(html[start..end].to_string());
    }

    None
}

fn has_class(fragment: &str, class_name: &str) -> bool {
    attr(open_tag(fragment), "class")
        .map(|classes| classes.split_whitespace().any(|class| class == class_name))
        .unwrap_or(false)
}

fn tag_name(open_tag: &str) -> Option<String> {
    let trimmed = open_tag.trim_start();
    let without_start = trimmed.strip_prefix('<')?;
    let name = without_start
        .chars()
        .skip_while(|ch| *ch == '/')
        .take_while(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn open_tag(fragment: &str) -> &str {
    fragment
        .find('>')
        .map(|end| &fragment[..=end])
        .unwrap_or(fragment)
}

fn starts_with_tag(fragment: &str, tag: &str) -> bool {
    fragment
        .trim_start()
        .to_lowercase()
        .starts_with(&format!("<{tag}"))
}

fn attr(open_tag: &str, name: &str) -> Option<String> {
    let lower = open_tag.to_lowercase();
    let needle = format!("{}=", name.to_lowercase());
    let index = lower.find(&needle)?;
    let value_start = index + needle.len();
    let bytes = open_tag.as_bytes();
    let quote = bytes.get(value_start).copied()?;
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    let value_start = value_start + 1;
    let value_end = open_tag[value_start..].find(quote as char)? + value_start;
    Some(html_decode(&open_tag[value_start..value_end]))
}

fn query_param(url: &str, name: &str) -> Option<String> {
    let query = url.split_once('?')?.1.split('#').next().unwrap_or_default();
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if html_decode(key) == name {
            return Some(percent_decode(&html_decode(value)));
        }
    }
    None
}

fn absolutize(base_url: &str, href: &str) -> String {
    let href = html_decode(href);
    if href.starts_with("http://") || href.starts_with("https://") {
        return href;
    }

    let origin = origin(base_url).unwrap_or_else(|| BASE_URL.to_string());
    if href.starts_with('/') {
        return format!("{origin}{href}");
    }

    let base_path = base_url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(base_url);
    let path_start = base_path.find('/').unwrap_or(base_path.len());
    let path = &base_path[path_start..];
    let directory = path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("");
    normalize_url_path(&origin, &format!("{directory}/{href}"))
}

fn origin(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let host = rest.split('/').next()?;
    Some(format!("{scheme}://{host}"))
}

fn normalize_url_path(origin: &str, path: &str) -> String {
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    format!("{origin}/{}", parts.join("/"))
}

fn strip_tags(html: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    html_decode(&output)
}

fn clean_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn html_decode(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

fn percent_decode(value: &str) -> String {
    let mut output = Vec::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&value[index + 1..index + 3], 16) {
                output.push(hex);
                index += 3;
                continue;
            }
        }
        output.push(if bytes[index] == b'+' {
            b' '
        } else {
            bytes[index]
        });
        index += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn choose_min(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    match (a, b) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}
