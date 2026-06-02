use crate::client::VvzClient;
use crate::models::{EventDetail, Field, Row, SearchResult};

pub trait CliClient {
    fn search_events(
        &self,
        query: &str,
        term_guid: Option<&str>,
        lang: &str,
    ) -> Result<Vec<SearchResult>, String>;
    fn event(
        &self,
        event: &str,
        term_guid: Option<&str>,
        lang: &str,
    ) -> Result<EventDetail, String>;
    fn events(
        &self,
        field: &str,
        term_guid: Option<&str>,
        lang: &str,
    ) -> Result<Vec<SearchResult>, String>;
    fn fields(&self, term_guid: Option<&str>, lang: &str) -> Result<Vec<Field>, String>;
}

impl CliClient for VvzClient {
    fn search_events(
        &self,
        query: &str,
        term_guid: Option<&str>,
        lang: &str,
    ) -> Result<Vec<SearchResult>, String> {
        VvzClient::search_events(self, query, term_guid, lang)
    }

    fn event(
        &self,
        event: &str,
        term_guid: Option<&str>,
        lang: &str,
    ) -> Result<EventDetail, String> {
        VvzClient::event(self, event, term_guid, lang)
    }

    fn events(
        &self,
        field: &str,
        term_guid: Option<&str>,
        lang: &str,
    ) -> Result<Vec<SearchResult>, String> {
        VvzClient::events(self, field, term_guid, lang)
    }

    fn fields(&self, term_guid: Option<&str>, lang: &str) -> Result<Vec<Field>, String> {
        VvzClient::fields(self, term_guid, lang)
    }
}

pub fn run_env() -> i32 {
    let args = std::env::args().collect::<Vec<_>>();
    let timeout = parse_global_timeout(&args).unwrap_or(20);
    let client = VvzClient::new("https://vvz.ruhr-uni-bochum.de", timeout);
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();

    match run_with_client(&client, &refs) {
        Ok(output) => {
            print!("{output}");
            0
        }
        Err(error) => {
            eprintln!("rub-vvz: {error}");
            2
        }
    }
}

pub fn run_with_client<C: CliClient>(client: &C, args: &[&str]) -> Result<String, String> {
    if args.len() <= 1 || args.iter().any(|arg| *arg == "-h" || *arg == "--help") {
        return Ok(help());
    }

    let command_index = args
        .iter()
        .position(|arg| matches!(*arg, "search" | "event" | "events" | "fields"))
        .ok_or_else(|| "Befehl fehlt. Nutze --help.".to_string())?;
    let command = args[command_index];
    let rest = &args[command_index + 1..];
    let options = Options::parse(rest);

    match command {
        "search" => {
            let query = options
                .positionals
                .first()
                .ok_or_else(|| "search braucht einen Suchbegriff.".to_string())?;
            let mut results =
                client.search_events(query, options.term_guid.as_deref(), &options.lang)?;
            results.truncate(options.limit.unwrap_or(20));
            Ok(if options.json {
                search_results_json(&results)
            } else {
                search_results_text(&results)
            })
        }
        "event" => {
            let event = options
                .positionals
                .first()
                .ok_or_else(|| "event braucht eine gguid oder URL.".to_string())?;
            let detail = client.event(event, options.term_guid.as_deref(), &options.lang)?;
            Ok(if options.json {
                event_detail_json(&detail)
            } else {
                event_detail_text(&detail)
            })
        }
        "events" => {
            let field = options
                .positionals
                .first()
                .ok_or_else(|| "events braucht eine Bereichs-gguid oder URL.".to_string())?;
            let mut results = client.events(field, options.term_guid.as_deref(), &options.lang)?;
            results.truncate(options.limit.unwrap_or(100));
            Ok(if options.json {
                search_results_json(&results)
            } else {
                search_results_text(&results)
            })
        }
        "fields" => {
            let fields = client.fields(options.term_guid.as_deref(), &options.lang)?;
            Ok(if options.json {
                fields_json(&fields)
            } else {
                fields_text(&fields)
            })
        }
        _ => Err(format!("Unbekannter Befehl: {command}")),
    }
}

#[derive(Debug)]
struct Options {
    json: bool,
    lang: String,
    term_guid: Option<String>,
    limit: Option<usize>,
    positionals: Vec<String>,
}

impl Options {
    fn parse(args: &[&str]) -> Self {
        let mut options = Self {
            json: false,
            lang: "de".to_string(),
            term_guid: None,
            limit: None,
            positionals: Vec::new(),
        };
        let mut index = 0;
        while index < args.len() {
            match args[index] {
                "--json" => options.json = true,
                "--lang" => {
                    if let Some(value) = args.get(index + 1) {
                        options.lang = (*value).to_string();
                        index += 1;
                    }
                }
                "--term-guid" => {
                    if let Some(value) = args.get(index + 1) {
                        options.term_guid = Some((*value).to_string());
                        index += 1;
                    }
                }
                "--limit" => {
                    if let Some(value) = args.get(index + 1) {
                        options.limit = value.parse().ok();
                        index += 1;
                    }
                }
                value if value.starts_with('-') => {}
                value => options.positionals.push(value.to_string()),
            }
            index += 1;
        }
        options
    }
}

fn parse_global_timeout(args: &[String]) -> Option<u64> {
    args.windows(2)
        .find(|window| window[0] == "--timeout")
        .and_then(|window| window[1].parse().ok())
}

fn help() -> String {
    "usage: rub-vvz [--timeout SECONDS] {search,event,events,fields} ...\n\n\
CLI fuer das oeffentliche Vorlesungsverzeichnis der RUB.\n\n\
commands:\n\
  search <query>        Veranstaltungen suchen\n\
  event <gguid|url>     Veranstaltungsdetails anzeigen\n\
  events <gguid|url>    Veranstaltungen eines VVZ-Bereichs listen\n\
  fields                VVZ-Bereiche/Fakultaeten listen\n\n\
options:\n\
  --json                JSON statt Text ausgeben\n\
  --term-guid <tguid>   Semester explizit setzen\n\
  --lang <lang>         VVZ-Sprache, Standard: de\n\
  --limit <n>           Ergebnislimit\n"
        .to_string()
}

fn search_results_text(results: &[SearchResult]) -> String {
    if results.is_empty() {
        return "Keine Veranstaltungen gefunden.\n".to_string();
    }
    let mut output = String::new();
    for (index, result) in results.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", index + 1, result.title));
        if let Some(guid) = &result.event_guid {
            output.push_str(&format!("   gguid: {guid}\n"));
        }
        if let Some(term_guid) = &result.term_guid {
            output.push_str(&format!("   tguid: {term_guid}\n"));
        }
        if !result.summary.is_empty() {
            output.push_str(&format!("   {}\n", result.summary));
        }
        output.push_str(&format!("   {}\n", result.url));
    }
    output
}

fn event_detail_text(detail: &EventDetail) -> String {
    let mut output = String::new();
    output.push_str(&format!("{}\n", detail.title));
    if let Some(guid) = &detail.event_guid {
        output.push_str(&format!("gguid: {guid}\n"));
    }
    if let Some(term_guid) = &detail.term_guid {
        output.push_str(&format!("tguid: {term_guid}\n"));
    }
    output.push_str(&format!("{}\n", detail.url));
    for (key, value) in &detail.fields {
        output.push_str(&format!("{key}: {value}\n"));
    }
    for (section, rows) in &detail.sections {
        output.push_str(&format!("\n{section}\n"));
        for row in rows {
            let cells = row
                .iter()
                .map(|(key, value)| format!("{key}: {value}"))
                .collect::<Vec<_>>()
                .join(" | ");
            output.push_str(&format!("- {cells}\n"));
        }
    }
    output
}

fn fields_text(fields: &[Field]) -> String {
    if fields.is_empty() {
        return "Keine VVZ-Bereiche gefunden.\n".to_string();
    }
    let mut output = String::new();
    for (index, field) in fields.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", index + 1, field.name));
        if let Some(value) = &field.value {
            output.push_str(&format!("   field: {value}\n"));
        }
        if let Some(guid) = &field.guid {
            output.push_str(&format!("   gguid: {guid}\n"));
        }
        if let Some(term_guid) = &field.term_guid {
            output.push_str(&format!("   tguid: {term_guid}\n"));
        }
        output.push_str(&format!("   {}\n", field.url));
    }
    output
}

fn search_results_json(results: &[SearchResult]) -> String {
    format!(
        "[{}]\n",
        results
            .iter()
            .map(search_result_json)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn search_result_json(result: &SearchResult) -> String {
    format!(
        "{{\"title\":\"{}\",\"url\":\"{}\",\"event_guid\":{},\"term_guid\":{},\"summary\":\"{}\"}}",
        json_escape(&result.title),
        json_escape(&result.url),
        option_json(result.event_guid.as_deref()),
        option_json(result.term_guid.as_deref()),
        json_escape(&result.summary)
    )
}

fn fields_json(fields: &[Field]) -> String {
    format!(
        "[{}]\n",
        fields.iter().map(field_json).collect::<Vec<_>>().join(",")
    )
}

fn field_json(field: &Field) -> String {
    format!(
        "{{\"name\":\"{}\",\"url\":\"{}\",\"value\":{},\"guid\":{},\"term_guid\":{}}}",
        json_escape(&field.name),
        json_escape(&field.url),
        option_json(field.value.as_deref()),
        option_json(field.guid.as_deref()),
        option_json(field.term_guid.as_deref())
    )
}

fn event_detail_json(detail: &EventDetail) -> String {
    format!(
        "{{\"title\":\"{}\",\"url\":\"{}\",\"event_guid\":{},\"term_guid\":{},\"fields\":{},\"sections\":{},\"description\":\"{}\"}}\n",
        json_escape(&detail.title),
        json_escape(&detail.url),
        option_json(detail.event_guid.as_deref()),
        option_json(detail.term_guid.as_deref()),
        string_map_json(&detail.fields),
        sections_json(&detail.sections),
        json_escape(&detail.description)
    )
}

fn string_map_json(map: &std::collections::BTreeMap<String, String>) -> String {
    format!(
        "{{{}}}",
        map.iter()
            .map(|(key, value)| format!("\"{}\":\"{}\"", json_escape(key), json_escape(value)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn sections_json(sections: &std::collections::BTreeMap<String, Vec<Row>>) -> String {
    format!(
        "{{{}}}",
        sections
            .iter()
            .map(|(section, rows)| {
                format!(
                    "\"{}\":[{}]",
                    json_escape(section),
                    rows.iter()
                        .map(string_map_json)
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn option_json(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", json_escape(value)))
        .unwrap_or_else(|| "null".to_string())
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch => escaped.push(ch),
        }
    }
    escaped
}
