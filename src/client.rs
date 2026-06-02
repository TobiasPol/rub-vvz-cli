use std::process::Command;

use crate::models::{EventDetail, Field, SearchResult};
use crate::parsing::{
    discover_term_guid, event_url, parse_event_detail, parse_fields, parse_search_results,
    url_encode,
};

const BASE_URL: &str = "https://vvz.ruhr-uni-bochum.de";
const EFFECTIVE_URL_MARKER: &str = "\n__RUB_VVZ_EFFECTIVE_URL__:";

#[derive(Debug, Clone)]
pub struct VvzClient {
    base_url: String,
    timeout_secs: u64,
}

impl Default for VvzClient {
    fn default() -> Self {
        Self::new(BASE_URL, 20)
    }
}

impl VvzClient {
    pub fn new(base_url: &str, timeout_secs: u64) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout_secs,
        }
    }

    pub fn current_term_guid(&self) -> Result<String, String> {
        let (final_url, html) = self.request("/", None)?;
        discover_term_guid(&html, &final_url)
            .ok_or_else(|| "Konnte keine tguid fuer das aktuelle VVZ-Semester finden.".to_string())
    }

    pub fn search_events(
        &self,
        query: &str,
        term_guid: Option<&str>,
        lang: &str,
    ) -> Result<Vec<SearchResult>, String> {
        let term_guid = self.term_guid(term_guid)?;
        let payload = format!(
            "find={}&searchobject=event&tguid={}&lang={}&searchButton=Suchen",
            url_encode(query),
            url_encode(&term_guid),
            url_encode(lang)
        );
        let (final_url, html) = self.request("/campus/all/dispatcher.asp", Some(&payload))?;
        Ok(parse_search_results(&html, &final_url))
    }

    pub fn event(
        &self,
        event_guid_or_url: &str,
        term_guid: Option<&str>,
        lang: &str,
    ) -> Result<EventDetail, String> {
        let url = if event_guid_or_url.starts_with("http://")
            || event_guid_or_url.starts_with("https://")
        {
            event_guid_or_url.to_string()
        } else {
            let term_guid = self.term_guid(term_guid)?;
            event_url(event_guid_or_url, &term_guid, lang)
        };
        let (final_url, html) = self.request(&url, None)?;
        Ok(parse_event_detail(&html, &final_url))
    }

    pub fn events(
        &self,
        field_guid_or_url: &str,
        term_guid: Option<&str>,
        lang: &str,
    ) -> Result<Vec<SearchResult>, String> {
        let url = if field_guid_or_url.starts_with("http://")
            || field_guid_or_url.starts_with("https://")
        {
            field_guid_or_url.to_string()
        } else {
            let term_guid = self.term_guid(term_guid)?;
            format!(
                "/campus/all/eventlist.asp?gguid={}&mode=field&tguid={}&lang={}",
                url_encode(field_guid_or_url),
                url_encode(&term_guid),
                url_encode(lang)
            )
        };
        let (final_url, html) = self.request(&url, None)?;
        Ok(parse_search_results(&html, &final_url))
    }

    pub fn fields(&self, term_guid: Option<&str>, lang: &str) -> Result<Vec<Field>, String> {
        let term_guid = self.term_guid(term_guid)?;
        let url = format!(
            "/campus/all/fields.asp?group=Vorlesungsverzeichnis&tguid={}&lang={}",
            url_encode(&term_guid),
            url_encode(lang)
        );
        let (final_url, html) = self.request(&url, None)?;
        Ok(parse_fields(&html, &final_url))
    }

    fn term_guid(&self, term_guid: Option<&str>) -> Result<String, String> {
        term_guid
            .map(ToOwned::to_owned)
            .map(Ok)
            .unwrap_or_else(|| self.current_term_guid())
    }

    fn request(&self, path_or_url: &str, data: Option<&str>) -> Result<(String, String), String> {
        let url = if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
            path_or_url.to_string()
        } else {
            format!("{}/{}", self.base_url, path_or_url.trim_start_matches('/'))
        };

        let mut command = Command::new("curl");
        command
            .arg("--http1.1")
            .arg("-L")
            .arg("-sS")
            .arg("--fail")
            .arg("--max-time")
            .arg(self.timeout_secs.to_string())
            .arg("-A")
            .arg("rub-vvz-cli/0.1")
            .arg("-w")
            .arg(format!("{EFFECTIVE_URL_MARKER}%{{url_effective}}"));

        if let Some(payload) = data {
            command
                .arg("-H")
                .arg("Content-Type: application/x-www-form-urlencoded")
                .arg("--data")
                .arg(payload);
        }

        let output = command
            .arg(&url)
            .output()
            .map_err(|error| format!("Konnte curl nicht starten. Ist curl installiert? {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "HTTP-Abruf fehlgeschlagen fuer {url}: {}",
                stderr.trim()
            ));
        }

        let marker = EFFECTIVE_URL_MARKER.as_bytes();
        let Some(marker_index) = find_last_marker(&output.stdout, marker) else {
            return Ok((url, decode_text(&output.stdout)));
        };
        let body = decode_text(&output.stdout[..marker_index]);
        let effective_url = String::from_utf8_lossy(&output.stdout[marker_index + marker.len()..])
            .trim()
            .to_string();
        Ok((effective_url, body))
    }
}

fn find_last_marker(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

fn decode_text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| {
        bytes
            .iter()
            .map(|byte| match *byte {
                0x80 => '\u{20AC}',
                0x82 => '\u{201A}',
                0x83 => '\u{0192}',
                0x84 => '\u{201E}',
                0x85 => '\u{2026}',
                0x86 => '\u{2020}',
                0x87 => '\u{2021}',
                0x88 => '\u{02C6}',
                0x89 => '\u{2030}',
                0x8A => '\u{0160}',
                0x8B => '\u{2039}',
                0x8C => '\u{0152}',
                0x8E => '\u{017D}',
                0x91 => '\u{2018}',
                0x92 => '\u{2019}',
                0x93 => '\u{201C}',
                0x94 => '\u{201D}',
                0x95 => '\u{2022}',
                0x96 => '\u{2013}',
                0x97 => '\u{2014}',
                0x98 => '\u{02DC}',
                0x99 => '\u{2122}',
                0x9A => '\u{0161}',
                0x9B => '\u{203A}',
                0x9C => '\u{0153}',
                0x9E => '\u{017E}',
                0x9F => '\u{0178}',
                byte => byte as char,
            })
            .collect()
    })
}
