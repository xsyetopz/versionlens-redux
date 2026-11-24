use super::*;

fn v(text: &str) -> Version {
    Version::parse(text).unwrap()
}

#[test]
fn replaces_in_the_users_style() {
    assert_eq!(replacement("^1.2.3", &v("1.4.0"), ""), "^1.4.0");
    assert_eq!(replacement("~> 1.2", &v("2.1.0"), ""), "~> 2.1");
    assert_eq!(replacement(">=4.2, <5", &v("5.1"), ""), ">=5.1");
    assert_eq!(replacement("v4", &v("v5.0.0"), ""), "v5");
    assert_eq!(replacement("", &v("2.0"), "=="), "==2.0");
    assert_eq!(replacement("[1.0,2.0)", &v("3.0"), ""), "[3.0]");
    assert_eq!(replacement("20-alpine", &v("22.1-alpine"), ""), "22-alpine");
}

#[test]
fn positions_count_utf16_units() {
    let text = "a\n€b c\n";
    let lines = Lines::new(text);
    let at = text.find('c').unwrap();
    assert_eq!(lines.position(at), Position::new(1, 3));
    assert_eq!(lines.position(0), Position::new(0, 0));
}

#[test]
fn shortens_errors_to_their_cause() {
    assert_eq!(
        short("https://x: error sending request: dns error"),
        "dns error"
    );
}

#[test]
fn maps_positions_to_byte_offsets() {
    let lines = Lines::new("a€b\nxy\n");
    assert_eq!(lines.offset(Position::new(0, 2)), 4);
    assert_eq!(lines.offset(Position::new(0, 99)), 5);
    assert_eq!(lines.offset(Position::new(1, 1)), 7);
    assert_eq!(lines.offset(Position::new(9, 0)), 9);
}

#[test]
fn offers_updates_only_to_matching_kinds() {
    let kinds = |list: &[&str]| {
        list.iter()
            .map(|k| CodeActionKind::from(k.to_string()))
            .collect::<Vec<_>>()
    };
    assert!(wants(ACTION_KIND, None));
    assert!(wants(ACTION_KIND, Some(&kinds(&["refactor"]))));
    assert!(wants(
        ACTION_KIND,
        Some(&kinds(&["quickfix", "refactor.rewrite"]))
    ));
    assert!(!wants(ACTION_KIND, Some(&kinds(&["quickfix"]))));
    assert!(!wants(ACTION_KIND, Some(&kinds(&["refactor.re"]))));
    assert!(!wants(
        ACTION_KIND,
        Some(&kinds(&["source.organizeImports"]))
    ));
}

#[test]
fn bulk_updates_pick_the_named_level() {
    let dep = Dep {
        name: "a".into(),
        req: "^1.2.0".into(),
        span: 0..6,
        registry: "npm",
        source: None,
    };
    let versions: Vec<Version> = ["1.2.0", "1.2.5", "1.4.0", "2.1.0"].map(v).into();
    let req = Req::parse("^1.2.0", version::Bare::Exact).unwrap();
    let view = View {
        dep: &dep,
        range: Range::default(),
        state: State::Planned {
            plan: Box::new(version::plan(&req, &versions, None, false).unwrap()),
            current: req.floor().cloned(),
        },
        vulns: &[],
        page: String::new(),
    };
    let text = |level| {
        updates(std::slice::from_ref(&view), "", level)
            .into_iter()
            .map(|e| e.new_text)
            .collect::<Vec<_>>()
    };
    assert_eq!(text("latest"), ["^2.1.0"]);
    assert_eq!(text("major"), ["^2.1.0"]);
    assert_eq!(text("minor"), ["^1.4.0"]);
    assert_eq!(text("patch"), ["^1.2.5"]);
}

#[test]
fn status_follows_the_original_wording() {
    let glyphs = Indicators::default();
    let plan = |resolved: Option<&str>| Plan {
        latest: v("2.1.0"),
        satisfied: false,
        resolved: resolved.map(v),
        picks: Vec::new(),
        steps: Vec::new(),
    };
    let (sat, latest, none) = (&glyphs.satisfies, &glyphs.latest, &glyphs.no_match);
    assert_eq!(
        status(&plan(None), false, &glyphs),
        format!("{none} no match")
    );
    assert_eq!(
        status(&plan(Some("1.4.0")), false, &glyphs),
        format!("{sat} satisfies 1.4.0")
    );
    assert_eq!(
        status(&plan(Some("2.1.0")), false, &glyphs),
        format!("{latest} satisfies latest 2.1.0")
    );
    assert_eq!(
        status(&plan(Some("2.1.0")), true, &glyphs),
        format!("{latest} latest 2.1.0")
    );
}
