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
    if args.len() <= 1 || args[1] == "-h" || args[1] == "--help" {
        return Ok(help());
    }

    let action_index = args.iter().position(|arg| is_command(arg));
    let help_index = args.iter().position(|arg| *arg == "help");

    if let Some(help_index) = help_index.filter(|help_index| {
        action_index
            .map(|action_index| *help_index < action_index)
            .unwrap_or(true)
    }) {
        let rest = &args[help_index + 1..];
        if rest.iter().any(|arg| is_help_flag(arg)) {
            return command_help("help");
        }
        return match rest.first().copied() {
            Some(command) => command_help(command),
            None => Ok(help()),
        };
    }

    if action_index.is_none() && args.iter().skip(1).any(|arg| is_help_flag(arg)) {
        return Ok(help());
    }
    let command_index =
        action_index.ok_or_else(|| "Missing command. Run 'rub-vvz help'.".to_string())?;
    let command = args[command_index];
    let rest = &args[command_index + 1..];
    if rest.iter().any(|arg| is_help_flag(arg)) {
        return command_help(command);
    }
    let options = Options::parse(rest)?;

    match command {
        "search" => {
            let query = options
                .positionals
                .first()
                .ok_or_else(|| "search requires a query.".to_string())?;
            let mut results =
                client.search_events(query, options.term_guid.as_deref(), &options.lang)?;
            results.truncate(options.limit.unwrap_or(20));
            Ok(if options.json {
                search_results_json(&results)
            } else {
                search_results_text(&results, ResultListKind::Search)
            })
        }
        "event" => {
            let event = options
                .positionals
                .first()
                .ok_or_else(|| "event requires a gguid or URL.".to_string())?;
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
                .ok_or_else(|| "events requires a field gguid or URL.".to_string())?;
            let mut results = client.events(field, options.term_guid.as_deref(), &options.lang)?;
            results.truncate(options.limit.unwrap_or(100));
            Ok(if options.json {
                search_results_json(&results)
            } else {
                search_results_text(&results, ResultListKind::Events)
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
        _ => Err(format!("Unknown command: {command}")),
    }
}

fn is_command(value: &str) -> bool {
    matches!(value, "search" | "event" | "events" | "fields")
}

fn is_help_flag(value: &str) -> bool {
    matches!(value, "-h" | "--help")
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
    fn parse(args: &[&str]) -> Result<Self, String> {
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
                value if value.starts_with("--lang=") => {
                    options.lang = value["--lang=".len()..].to_string();
                }
                "--lang" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| "--lang requires a value.".to_string())?;
                    options.lang = (*value).to_string();
                    index += 1;
                }
                value if value.starts_with("--term-guid=") => {
                    options.term_guid = Some(value["--term-guid=".len()..].to_string());
                }
                "--term-guid" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| "--term-guid requires a value.".to_string())?;
                    options.term_guid = Some((*value).to_string());
                    index += 1;
                }
                value if value.starts_with("--limit=") => {
                    options.limit = Some(parse_limit(&value["--limit=".len()..])?);
                }
                "--limit" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| "--limit requires a value.".to_string())?;
                    options.limit = Some(parse_limit(value)?);
                    index += 1;
                }
                value if value.starts_with('-') => {
                    return Err(format!("Unknown option: {value}. Run 'rub-vvz help'."));
                }
                value => options.positionals.push(value.to_string()),
            }
            index += 1;
        }
        Ok(options)
    }
}

fn parse_global_timeout(args: &[String]) -> Option<u64> {
    args.iter()
        .find_map(|arg| arg.strip_prefix("--timeout="))
        .and_then(|value| value.parse().ok())
        .or_else(|| {
            args.windows(2)
                .find(|window| window[0] == "--timeout")
                .and_then(|window| window[1].parse().ok())
        })
}

fn parse_limit(value: &str) -> Result<usize, String> {
    match value.parse::<usize>() {
        Ok(limit) if limit > 0 => Ok(limit),
        _ => Err(format!("--limit requires a positive number, got: {value}")),
    }
}

fn help() -> String {
    concat!(
        "Usage: rub-vvz [--timeout SECONDS] COMMAND [ARGS]...\n\n",
        "CLI for Ruhr-Universitaet Bochum's public course catalogue.\n\n",
        "Commands:\n",
        "  search      Search public courses\n",
        "  event       Show course details\n",
        "  events      List courses in a VVZ area\n",
        "  fields      List VVZ areas and faculties\n",
        "  help        Show help, optionally for a command\n\n",
        "Global Options:\n",
        "  --timeout SECONDS   HTTP timeout, default: 20\n",
        "  -h, --help          Show help\n\n",
        "Output:\n",
        "  Text output explains gguid, tguid, and the next useful command.\n",
        "  Use --json for stable machine-readable output without explanations.\n\n",
        "Examples:\n",
        "  rub-vvz search \"Software Engineering\" --limit 5\n",
        "  rub-vvz event 0xEVENT --term-guid 0xTERM\n",
        "  rub-vvz help search\n"
    )
    .to_string()
}

fn command_help(command: &str) -> Result<String, String> {
    match command {
        "search" => Ok(concat!(
            "Usage: rub-vvz search <query> [OPTIONS]\n\n",
            "Searches public VVZ courses by title, lecturer, or course number.\n\n",
            "Options:\n",
            "  --json                Print a JSON array instead of explained text\n",
            "  --limit N             Result limit, default: 20\n",
            "  --term-guid TGUID     Set the semester explicitly\n",
            "  --lang LANG           VVZ language, default: de\n",
            "  -h, --help            Show help for search\n\n",
            "Output:\n",
            "  Text: numbered results with title, gguid, tguid, summary, and URL.\n",
            "  JSON: array with title, url, event_guid, term_guid, and summary.\n",
            "  Next step: rub-vvz event <gguid> --term-guid <tguid>\n\n",
            "Examples:\n",
            "  rub-vvz search \"Software Engineering\" --limit 5\n",
            "  rub-vvz search \"Software Engineering\" --json\n"
        )
            .to_string()),
        "event" => Ok(concat!(
            "Usage: rub-vvz event <gguid|url> [OPTIONS]\n\n",
            "Shows details for a public VVZ course. The argument can be a raw gguid\n",
            "or a complete event.asp URL.\n\n",
            "Options:\n",
            "  --json                Print a JSON object instead of explained text\n",
            "  --term-guid TGUID     Set the semester, important for raw gguid values\n",
            "  --lang LANG           VVZ language, default: de\n",
            "  -h, --help            Show help for event\n\n",
            "Output:\n",
            "  Text: title, identifiers, CampusOffice fields, table sections, and description.\n",
            "  JSON: object with title, url, event_guid, term_guid, fields, sections, and description.\n\n",
            "Examples:\n",
            "  rub-vvz event 0xEVENT --term-guid 0xTERM\n",
            "  rub-vvz event \"https://vvz.ruhr-uni-bochum.de/campus/all/event.asp?...\" --json\n"
        )
            .to_string()),
        "events" => Ok(concat!(
            "Usage: rub-vvz events <field-gguid|url> [OPTIONS]\n\n",
            "Lists public courses in a VVZ area or faculty.\n\n",
            "Options:\n",
            "  --json                Print a JSON array instead of explained text\n",
            "  --limit N             Result limit, default: 100\n",
            "  --term-guid TGUID     Set the semester explicitly\n",
            "  --lang LANG           VVZ language, default: de\n",
            "  -h, --help            Show help for events\n\n",
            "Output:\n",
            "  Text: numbered courses with title, gguid, tguid, summary, and URL.\n",
            "  JSON: array with title, url, event_guid, term_guid, and summary.\n",
            "  Next step: rub-vvz event <gguid> --term-guid <tguid>\n\n",
            "Examples:\n",
            "  rub-vvz events 0xFIELD --term-guid 0xTERM --limit 10\n",
            "  rub-vvz events \"https://vvz.ruhr-uni-bochum.de/campus/all/eventlist.asp?...\" --json\n"
        )
            .to_string()),
        "fields" => Ok(concat!(
            "Usage: rub-vvz fields [OPTIONS]\n\n",
            "Lists top-level VVZ areas and faculties for the selected semester.\n\n",
            "Options:\n",
            "  --json                Print a JSON array instead of explained text\n",
            "  --term-guid TGUID     Set the semester explicitly\n",
            "  --lang LANG           VVZ language, default: de\n",
            "  -h, --help            Show help for fields\n\n",
            "Output:\n",
            "  Text: area names with field, gguid, tguid, and URL.\n",
            "  JSON: array with name, url, value, guid, and term_guid.\n",
            "  Next step: rub-vvz events <gguid> --term-guid <tguid>\n\n",
            "Examples:\n",
            "  rub-vvz fields\n",
            "  rub-vvz fields --json\n"
        )
            .to_string()),
        "help" => Ok(concat!(
            "Usage: rub-vvz help [COMMAND]\n\n",
            "Shows root help or help for search, event, events, or fields.\n\n",
            "Examples:\n",
            "  rub-vvz help\n",
            "  rub-vvz help search\n"
        )
            .to_string()),
        value => Err(format!(
            "Unknown help topic: {value}. Run 'rub-vvz help'."
        )),
    }
}

enum ResultListKind {
    Search,
    Events,
}

fn search_results_text(results: &[SearchResult], kind: ResultListKind) -> String {
    let (label, empty_hint) = match kind {
        ResultListKind::Search => (
            "rub-vvz search",
            "The query returned no public VVZ courses for the selected semester. Check the spelling or set --term-guid for another semester.",
        ),
        ResultListKind::Events => (
            "rub-vvz events",
            "The VVZ area returned no public courses for the selected semester. Check the area gguid or set --term-guid.",
        ),
    };
    if results.is_empty() {
        return format!("{label}: 0 courses found.\n\nInterpretation: {empty_hint}\n");
    }
    let mut output = String::new();
    output.push_str(&format!("{label}: {} courses found.\n\n", results.len()));
    output.push_str(
        "Interpretation: gguid is the course identifier and tguid is the semester. \
Run 'rub-vvz event <gguid> --term-guid <tguid>' for details or use '--json' \
for machine-readable output.\n\n",
    );
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
    output.push_str("rub-vvz event: course details\n\n");
    output.push_str(
        "Interpretation: gguid is the course identifier and tguid is the semester. \
Fields are CampusOffice metadata; sections contain tables such as appointments, \
lecturers, modules, or audiences. Use '--json' for machine-readable output.\n\n",
    );
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
    if !detail.description.trim().is_empty() {
        output.push_str(&format!("\nDescription\n{}\n", detail.description.trim()));
    }
    output
}

fn fields_text(fields: &[Field]) -> String {
    if fields.is_empty() {
        return "rub-vvz fields: 0 VVZ areas found.\n\nInterpretation: No public VVZ areas were detected for the selected semester. Check --term-guid or RUB VVZ availability.\n".to_string();
    }
    let mut output = String::new();
    output.push_str(&format!(
        "rub-vvz fields: {} VVZ areas found.\n\n",
        fields.len()
    ));
    output.push_str(
        "Interpretation: gguid is the area identifier and tguid is the semester. \
Run 'rub-vvz events <gguid> --term-guid <tguid>' for courses in an area or \
use '--json' for machine-readable output.\n\n",
    );
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
