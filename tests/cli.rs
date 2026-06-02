use rub_vvz::cli::{run_with_client, CliClient};
use rub_vvz::models::{EventDetail, Field, SearchResult};

struct FakeClient {
    expected_query: &'static str,
    expected_term_guid: Option<&'static str>,
    expected_lang: &'static str,
}

impl Default for FakeClient {
    fn default() -> Self {
        Self {
            expected_query: "Cryptography",
            expected_term_guid: None,
            expected_lang: "de",
        }
    }
}

impl CliClient for FakeClient {
    fn search_events(
        &self,
        query: &str,
        term_guid: Option<&str>,
        lang: &str,
    ) -> Result<Vec<SearchResult>, String> {
        assert_eq!(query, self.expected_query);
        assert_eq!(term_guid, self.expected_term_guid);
        assert_eq!(lang, self.expected_lang);
        Ok(vec![SearchResult {
            title: "100001 Cryptography".to_string(),
            url: "https://example.test/event.asp?gguid=0xEVENT&tguid=0xTERM".to_string(),
            event_guid: Some("0xEVENT".to_string()),
            term_guid: Some("0xTERM".to_string()),
            summary: "Lecture".to_string(),
        }])
    }

    fn event(&self, _: &str, _: Option<&str>, _: &str) -> Result<EventDetail, String> {
        unimplemented!()
    }

    fn events(&self, _: &str, _: Option<&str>, _: &str) -> Result<Vec<SearchResult>, String> {
        unimplemented!()
    }

    fn fields(&self, _: Option<&str>, _: &str) -> Result<Vec<Field>, String> {
        unimplemented!()
    }
}

#[test]
fn help_command_prints_root_help() {
    let output = run_with_client(&FakeClient::default(), &["rub-vvz", "help"])
        .expect("cli should print help");

    assert!(output.contains("Usage: rub-vvz [--timeout SECONDS] COMMAND [ARGS]..."));
    assert!(output.contains("rub-vvz help search"));
}

#[test]
fn help_command_prints_command_help() {
    let output = run_with_client(&FakeClient::default(), &["rub-vvz", "help", "search"])
        .expect("cli should print command help");

    assert!(output.contains("Usage: rub-vvz search <query> [OPTIONS]"));
    assert!(output.contains("Next step: rub-vvz event <gguid> --term-guid <tguid>"));
}

#[test]
fn help_command_after_global_options_prints_command_help() {
    let output = run_with_client(
        &FakeClient::default(),
        &["rub-vvz", "--timeout", "5", "help", "search"],
    )
    .expect("cli should print command help");

    assert!(output.contains("Usage: rub-vvz search <query> [OPTIONS]"));
}

#[test]
fn command_help_flag_prints_command_help() {
    let output = run_with_client(&FakeClient::default(), &["rub-vvz", "search", "--help"])
        .expect("cli should print command help");

    assert!(output.contains("Usage: rub-vvz search <query> [OPTIONS]"));
}

#[test]
fn global_help_flag_after_global_options_prints_root_help() {
    let output = run_with_client(
        &FakeClient::default(),
        &["rub-vvz", "--timeout", "5", "--help"],
    )
    .expect("cli should print root help");

    assert!(output.contains("Global Options:"));
}

#[test]
fn search_prints_explained_text_results() {
    let output = run_with_client(
        &FakeClient::default(),
        &["rub-vvz", "search", "Cryptography"],
    )
    .expect("cli should run");

    assert!(output.contains("rub-vvz search: 1 courses found."));
    assert!(output.contains("Interpretation: gguid is the course identifier"));
    assert!(output.contains("rub-vvz event <gguid> --term-guid <tguid>"));
    assert!(output.contains("100001 Cryptography"));
}

#[test]
fn help_can_be_a_search_query() {
    let client = FakeClient {
        expected_query: "help",
        expected_term_guid: None,
        expected_lang: "de",
    };
    let output =
        run_with_client(&client, &["rub-vvz", "search", "help"]).expect("cli should run search");

    assert!(output.contains("rub-vvz search: 1 courses found."));
}

#[test]
fn search_prints_json_results() {
    let output = run_with_client(
        &FakeClient::default(),
        &["rub-vvz", "search", "Cryptography", "--json"],
    )
    .expect("cli should run");

    assert!(output.contains(r#""title":"100001 Cryptography""#));
    assert!(output.contains(r#""event_guid":"0xEVENT""#));
    assert!(!output.contains("Interpretation:"));
}

#[test]
fn equals_style_options_are_parsed() {
    let client = FakeClient {
        expected_query: "Cryptography",
        expected_term_guid: Some("0xTERM"),
        expected_lang: "en",
    };
    let output = run_with_client(
        &client,
        &[
            "rub-vvz",
            "search",
            "Cryptography",
            "--json",
            "--limit=1",
            "--term-guid=0xTERM",
            "--lang=en",
        ],
    )
    .expect("cli should run");

    assert!(output.contains(r#""term_guid":"0xTERM""#));
}

#[test]
fn unknown_options_fail_instead_of_being_ignored() {
    let error = run_with_client(
        &FakeClient::default(),
        &["rub-vvz", "search", "Cryptography", "--jsoon"],
    )
    .expect_err("unknown flags should fail");

    assert!(error.contains("Unknown option: --jsoon"));
}
