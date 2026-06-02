use rub_vvz::cli::{run_with_client, CliClient};
use rub_vvz::models::{EventDetail, Field, SearchResult};

#[derive(Default)]
struct FakeClient;

impl CliClient for FakeClient {
    fn search_events(
        &self,
        query: &str,
        term_guid: Option<&str>,
        lang: &str,
    ) -> Result<Vec<SearchResult>, String> {
        assert_eq!(query, "Kryptographie");
        assert_eq!(term_guid, None);
        assert_eq!(lang, "de");
        Ok(vec![SearchResult {
            title: "100001 Kryptographie".to_string(),
            url: "https://example.test/event.asp?gguid=0xEVENT&tguid=0xTERM".to_string(),
            event_guid: Some("0xEVENT".to_string()),
            term_guid: Some("0xTERM".to_string()),
            summary: "Vorlesung".to_string(),
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
fn search_prints_json_results() {
    let output = run_with_client(
        &FakeClient,
        &["rub-vvz", "search", "Kryptographie", "--json"],
    )
    .expect("cli should run");

    assert!(output.contains(r#""title":"100001 Kryptographie""#));
    assert!(output.contains(r#""event_guid":"0xEVENT""#));
}
