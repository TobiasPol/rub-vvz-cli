use rub_vvz::parsing::{
    discover_term_guid, parse_event_detail, parse_fields, parse_search_results,
};

const SEARCH_HTML: &str = r#"
<html>
  <body>
    <a href="/campus/all/event.asp?gguid=0xEVENT&amp;tguid=0xTERM&amp;lang=de">100001 Kryptographie</a>
    <span>Vorlesung, 4 SWS</span>
  </body>
</html>
"#;

const EVENTLIST_TABLE_HTML: &str = r#"
<html>
  <body>
    <table>
      <tr>
        <td id="cas-table_EVENTLIST_COURSENUMBER_1">
          <a href="event.asp?gguid=0xEVENT&amp;tguid=0xTERM&amp;lang=de">211009</a>
        </td>
        <td id="cas-table_EVENTLIST_TITLE_1">
          <a href="event.asp?gguid=0xEVENT&amp;tguid=0xTERM&amp;lang=de">Einführung in die Kryptographie 2</a>
        </td>
        <td id="cas-table_EVENTLIST_LECTURER_1">Paar, Christof</td>
        <td id="cas-table_EVENTLIST_SWS_1">Vorlesung mit Übung</td>
      </tr>
    </table>
  </body>
</html>
"#;

const DISPLAYFIELDS_HTML: &str = r#"
<html>
  <head><title>eCampus</title></head>
  <body>
    <table class="displayfields">
      <tr>
        <td class="label"><span class="label-wrap">Lehrveranstaltungsnummer:</span></td>
        <td class="element"><div class="cas-extended-field-value">211009</div></td>
      </tr>
      <tr>
        <td class="label"><span class="label-wrap">Titel:</span></td>
        <td class="element"><div class="cas-extended-field-value">Einführung in die Kryptographie 2</div></td>
      </tr>
      <tr>
        <td class="label"><span class="label-wrap">Semesterwochenstunden [SWS]:</span></td>
        <td class="element"><div class="cas-extended-field-value">4</div></td>
      </tr>
    </table>
    <h2>Dozenten</h2>
    <table>
      <tr><th>Name</th><th>Einrichtungen</th></tr>
      <tr><td>Prof. Ada Lovelace</td><td>Informatik</td></tr>
    </table>
  </body>
</html>
"#;

const REAL_DISPLAYFIELDS_HTML: &str = r#"
<html>
  <head><title>eCampus</title></head>
  <body>
    <h2>Information</h2>
    <table class="DEBUG_1 displayfields">
      <tr>
        <td class="label">
          <span class="help-label-wrap">
            <span style="display: none;" class="tooltip">Die Lehrveranstaltungsnummer wird gedruckt.</span>
            <img class="help-icon" src="help.png" alt="Tooltip Help"></img>
            <span class="label-wrap">Lehrveranstaltungsnummer:</span>
          </span>
        </td>
        <td class="element" id="rwev_coursenumber">
          <div class="cas-extended-field-value">211009</div>
        </td>
      </tr>
      <tr>
        <td class="label">
          <span class="help-label-wrap">
            <span style="display: none;" class="tooltip">Der Titel wird gedruckt.</span>
            <img class="help-icon" src="help.png" alt="Tooltip Help"></img>
            <span class="label-wrap">Titel:</span>
          </span>
        </td>
        <td class="element">
          <div class="cas-extended-field-value">Einführung in die Kryptographie 2</div>
        </td>
      </tr>
    </table>
  </body>
</html>
"#;

const FIELDS_HTML: &str = r#"
<html>
  <body>
    <a href="subfields.asp?gguid=0xFIELD&amp;tguid=0xTERM&amp;lang=de">XXI. Fakultät für Informatik</a>
  </body>
</html>
"#;

#[test]
fn discovers_term_guid_from_url_or_html() {
    assert_eq!(
        discover_term_guid(
            "",
            "https://vvz.ruhr-uni-bochum.de/campus/all/groups.asp?tguid=0xTERM&lang=de"
        ),
        Some("0xTERM".to_string())
    );
    assert_eq!(
        discover_term_guid(SEARCH_HTML, "https://vvz.ruhr-uni-bochum.de/"),
        Some("0xTERM".to_string())
    );
}

#[test]
fn parses_search_results() {
    let results = parse_search_results(
        SEARCH_HTML,
        "https://vvz.ruhr-uni-bochum.de/campus/all/search.asp",
    );

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "100001 Kryptographie");
    assert_eq!(results[0].event_guid.as_deref(), Some("0xEVENT"));
}

#[test]
fn parses_eventlist_rows_as_single_result() {
    let results = parse_search_results(
        EVENTLIST_TABLE_HTML,
        "https://vvz.ruhr-uni-bochum.de/campus/all/dispatcher.asp",
    );

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Einführung in die Kryptographie 2");
    assert!(results[0].summary.contains("211009"));
    assert!(results[0].summary.contains("Paar, Christof"));
    assert!(results[0].summary.contains("Vorlesung mit Übung"));
}

#[test]
fn parses_event_detail_fields_and_sections() {
    let detail = parse_event_detail(
        DISPLAYFIELDS_HTML,
        "https://vvz.ruhr-uni-bochum.de/campus/all/event.asp?gguid=0xEVENT&tguid=0xTERM&lang=de",
    );

    assert_eq!(detail.title, "Einführung in die Kryptographie 2");
    assert_eq!(
        detail
            .fields
            .get("Lehrveranstaltungsnummer")
            .map(String::as_str),
        Some("211009")
    );
    assert_eq!(
        detail
            .fields
            .get("Semesterwochenstunden [SWS]")
            .map(String::as_str),
        Some("4")
    );
    assert_eq!(detail.sections["Dozenten"][0]["Name"], "Prof. Ada Lovelace");
}

#[test]
fn parses_visible_labels_when_help_label_wrap_is_present() {
    let detail = parse_event_detail(
        REAL_DISPLAYFIELDS_HTML,
        "https://vvz.ruhr-uni-bochum.de/campus/all/event.asp?gguid=0xEVENT&tguid=0xTERM&lang=de",
    );

    assert_eq!(detail.title, "Einführung in die Kryptographie 2");
    assert_eq!(
        detail
            .fields
            .get("Lehrveranstaltungsnummer")
            .map(String::as_str),
        Some("211009")
    );
    assert_eq!(
        detail.fields.get("Titel").map(String::as_str),
        Some("Einführung in die Kryptographie 2")
    );
}

#[test]
fn parses_public_subfield_links_as_fields() {
    let fields = parse_fields(
        FIELDS_HTML,
        "https://vvz.ruhr-uni-bochum.de/campus/all/fields.asp",
    );

    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].name, "XXI. Fakultät für Informatik");
    assert_eq!(fields[0].guid.as_deref(), Some("0xFIELD"));
    assert_eq!(fields[0].term_guid.as_deref(), Some("0xTERM"));
}
