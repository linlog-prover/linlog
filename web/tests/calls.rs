// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The calls as the page makes them, under WebAssembly in Node and as
//! ordinary tests elsewhere.

use linlog_web::calls::{self, StepForm};
use linlog_web::{Failure, tab};
use serde_json::{Value, json};
use wasm_bindgen_test::wasm_bindgen_test;

/// The tab's settings with some keys replaced.
fn settings(over: &Value) -> String {
    over.to_string()
}

/// Reads a call's JSON.
fn value(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

/// Reads a failure's JSON.
fn failure(failure: &Failure) -> Value {
    value(&failure.0)
}

/// A step's JSON.
fn step(position: usize, rule: &str, left: Option<Vec<usize>>) -> String {
    serde_json::to_string(&StepForm {
        position,
        rule: rule.into(),
        left,
        witness: None,
    })
    .unwrap()
}

/// A quantifier step's JSON, its witness given as text or left open.
fn witnessed(position: usize, rule: &str, witness: Option<&str>) -> String {
    serde_json::to_string(&StepForm {
        position,
        rule: rule.into(),
        left: None,
        witness: witness.map(Into::into),
    })
    .unwrap()
}

/// The position in a goal of the view of the formula written `text`.
fn at(answer: &str, goal: u32, text: &str) -> usize {
    let view = value(answer);
    let view = if view["view"].is_null() {
        view
    } else {
        view["view"].clone()
    };
    let goal = view["goals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["goal"] == goal)
        .unwrap_or_else(|| panic!("no goal {goal} in {view}"))
        .clone();
    let formulas = goal["formulas"].as_array().unwrap();
    formulas
        .iter()
        .position(|f| f["text"] == text)
        .unwrap_or_else(|| panic!("no {text} in {goal}"))
}

/// The open goals of a call's answer.
fn open_goals(answer: &str) -> Vec<u32> {
    let view = value(answer);
    let view = if view["view"].is_null() {
        view
    } else {
        view["view"].clone()
    };
    view["goals"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["goal"].as_u64().unwrap() as u32)
        .collect()
}

/// The session of a call's answer, as the page keeps it.
fn session(answer: &str) -> String {
    value(answer)["session"].to_string()
}

/// A proof by clicking: the rules a formula takes, a split, the axioms,
/// the finished proof checked and every export of it; the view says
/// which goals are open and what each drawn inference is.
#[wasm_bindgen_test(unsupported = test)]
fn a_proof_by_steps_is_checked_and_exported() {
    // ⊢ ~A, A ⊗ ~B, B: the goal's formulas are 0: ~A, 1: A ⊗ ~B, 2: B.
    let started = calls::start("A, A -o B |- B", "classical", "").unwrap();
    let view = &value(&started)["view"];
    assert_eq!(view["fragment"], "MLL");
    assert_eq!(view["goals"][0]["formulas"][1]["text"], "A ⊗ ~B");
    assert_eq!(view["net"], Value::Null);
    let root = view["goals"][0]["goal"].as_u64().unwrap() as u32;
    let state = session(&started);
    let rules = value(&calls::rules(&state, root, 1, "").unwrap());
    assert_eq!(rules, json!([{"rule": "⊗", "split": true}]));
    let split = calls::apply(&state, root, &step(1, "⊗", Some(vec![0])), "").unwrap();
    let opened = value(&split)["opened"].clone();
    let mut state = session(&split);
    for goal in opened.as_array().unwrap() {
        let goal = goal.as_u64().unwrap() as u32;
        state = session(&calls::apply(&state, goal, &step(0, "ax", None), "").unwrap());
    }
    let view = value(&calls::view(&state, "").unwrap());
    assert_eq!(
        (view["complete"].clone(), view["steps"].clone()),
        (json!(true), json!(3))
    );
    assert_eq!(
        value(&calls::proof(&state, "").unwrap())["nodes"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let drawing = calls::draw(&state, "").unwrap();
    assert!(drawing.contains("id=\"i0\"") && drawing.contains("id=\"i0-0\""));
    assert!(calls::draw_net(&state, "").unwrap().contains("id=\"l"));
    for (format, mark) in [
        ("text", "⊗"),
        ("latex", "\\otimes"),
        ("typst", "#prooftree"),
        ("rocq", "Proof."),
    ] {
        let text = calls::export(&state, format, "").unwrap();
        assert!(text.contains(mark), "{format}: {text}");
    }
}

/// A typed classical sequent's session keeps how it was written, and with
/// `view.classical` `"two"` its derivation is drawn two-sided in every
/// format, a negation step where a negation was written; by default it
/// stays one-sided.
#[wasm_bindgen_test(unsupported = test)]
fn a_classical_session_is_drawn_as_written() {
    let started = calls::start("A par B |- ~(~A * ~B)", "classical", "").unwrap();
    let state = session(&calls::close_all(&session(&started), "").unwrap());
    let two = settings(&json!({"view": {"classical": "two"}}));
    let tree = calls::export(&state, "text", &two).unwrap();
    assert!(
        tree.ends_with("A ⅋ B ⊢ ~(~A ⊗ ~B)") && tree.contains("~L"),
        "{tree}"
    );
    let one = calls::export(&state, "text", "").unwrap();
    assert!(
        one.ends_with("⊢ ~A ⊗ ~B, A ⅋ B") && !one.contains("~R"),
        "{one}"
    );
    for (format, mark) in [
        ("latex", r"\textasciitilde}R"),
        ("typst", r#""~R""#),
        ("svg", "by ~R"),
    ] {
        let text = calls::export(&state, format, &two).unwrap();
        assert!(text.contains(mark), "{format}: {text}");
    }
    assert!(calls::draw(&state, &two).unwrap().contains("by ~L"));
}

/// A cut opens two goals, the formula typed as written proved by the
/// first and used by the second; the finished proof has its cut, written
/// at the wire level that has it, and is drawn as written with the cut
/// named; the view gives each cut's formula as typed, with which a
/// session started on the sequent (its cut formulas left out) makes the
/// same cut again; text that is no formula of the mode is refused, the
/// session unchanged, and `undo` takes the cut back.
#[wasm_bindgen_test(unsupported = test)]
fn a_cut_opens_two_goals() {
    let started = calls::start("A, A -o B |- B", "classical", "").unwrap();
    let root = value(&started)["view"]["goals"][0]["goal"]
        .as_u64()
        .unwrap() as u32;
    let state = session(&started);
    let cut = calls::cut(&state, root, "A -o B", "[1]", "").unwrap();
    let replays = |cut: &str, mode: &str, left: &str, typed: &str| {
        let made = value(cut);
        assert_eq!(made["view"]["cuts"][0]["formula"], typed);
        let sequent = made["session"]["sequent"].to_string();
        let again = calls::start(&sequent, mode, "").unwrap();
        let replayed = calls::cut(&session(&again), root, typed, left, "").unwrap();
        let replayed = value(&replayed)["session"].clone();
        assert_eq!(
            (&replayed["sequent"], &replayed["inferences"]),
            (&made["session"]["sequent"], &made["session"]["inferences"])
        );
    };
    replays(&cut, "classical", "[1]", "A ⊸ B");
    let goals = &value(&cut)["view"]["goals"];
    let texts = |g: usize| -> Vec<Value> {
        goals[g]["formulas"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["text"].clone())
            .collect()
    };
    assert_eq!(texts(0), [json!("A ⊗ ~B"), json!("~A ⅋ B")]);
    assert_eq!(texts(1), [json!("~A"), json!("B"), json!("A ⊗ ~B")]);
    let closed = session(&calls::close_all(&session(&cut), "").unwrap());
    let proof = value(&calls::proof(&closed, "").unwrap());
    assert_eq!(proof["version"], 6);
    assert!(
        proof["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n.get("cut").is_some())
    );
    let two = settings(&json!({"view": {"classical": "two"}}));
    let tree = calls::export(&closed, "text", &two).unwrap();
    assert!(
        tree.contains("A ⊸ B ⊢ A ⊸ B") && tree.contains(" cut\n"),
        "{tree}"
    );
    let ill = calls::start("A, A -o B, B -o C |- C", "intuitionistic", "").unwrap();
    let refused = failure(&calls::cut(&session(&ill), root, "B par C", "[0, 1]", "").unwrap_err());
    assert_eq!(refused["code"], "parse");
    let cut = calls::cut(&session(&ill), root, "B", "[0, 1]", "").unwrap();
    replays(&cut, "intuitionistic", "[0, 1]", "B");
    let readings: Vec<Value> = value(&cut)["view"]["goals"][1]["formulas"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["reading"].clone())
        .collect();
    assert_eq!(readings, [json!("B ⊸ C"), json!("C"), json!("B")]);
    let undone = value(&calls::undo(&session(&cut), "").unwrap());
    assert_eq!(undone["session"], value(&ill)["session"]);
}

/// A finished proof with exponentials has its net with boxes: the view
/// gives no refusal, and `draw_net` draws each box a rectangle, with its
/// formulas or compact, in intuitionistic mode too, there also as its
/// essential net; in affine mode the view says why there is no net, and
/// `draw_net` refuses the same. The settings' `net_look` draws it in the
/// textbooks' look, by the preset's name or its fields.
#[wasm_bindgen_test(unsupported = test)]
fn a_proof_with_exponentials_draws_its_net_with_boxes() {
    let started = calls::start("!a, !b |- !a", "classical", "").unwrap();
    let state = session(&calls::close_all(&session(&started), "").unwrap());
    let view = value(&calls::view(&state, "").unwrap());
    assert_eq!(
        (view["complete"].clone(), view["net"].clone()),
        (json!(true), Value::Null)
    );
    let drawing = calls::draw_net(&state, "").unwrap();
    assert!(drawing.contains(r#"<path id="b2" d="M"#), "{drawing}");
    assert!(drawing.contains(r#"id="l"#), "{drawing}");
    let compact = settings(&json!({"styles": {"svg": {"net": "compact"}}}));
    assert!(
        calls::draw_net(&state, &compact)
            .unwrap()
            .contains("<circle")
    );
    // The textbooks' look, by its name or by its fields: the weakening of
    // `?b⊥` a node `?w` of its own.
    assert!(!drawing.contains(">?w<"), "{drawing}");
    let named = settings(&json!({"styles": {"svg": {"net_look": "textbook"}}}));
    let fields = settings(&json!({"styles": {"svg": {"net_look": {
        "premises": "plain", "conclusions": "bottom", "why_not": "rules"
    }}}}));
    let textbook = calls::draw_net(&state, &named).unwrap();
    assert!(textbook.contains(">?w<"), "{textbook}");
    assert_eq!(calls::draw_net(&state, &fields).unwrap(), textbook);

    // An intuitionistic proof's net with boxes, and its essential net;
    // affine logic has none.
    let started = calls::start("!a |- !a * !a", "intuitionistic", "").unwrap();
    let state = session(&calls::close_all(&session(&started), "").unwrap());
    let view = value(&calls::view(&state, "").unwrap());
    assert_eq!(
        (view["complete"].clone(), view["net"].clone()),
        (json!(true), Value::Null)
    );
    let drawing = calls::draw_net(&state, "").unwrap();
    assert!(drawing.contains(r#"<path id="b"#), "{drawing}");
    let essential = settings(&json!({"styles": {"svg": {"essential": true}}}));
    let read = calls::draw_net(&state, &essential).unwrap();
    assert!(read.contains(r##"stroke="#009e73""##), "{read}");
    let started = calls::start("!a |- !a", "intuitionistic-affine", "").unwrap();
    let state = session(&calls::close_all(&session(&started), "").unwrap());
    let view = value(&calls::view(&state, "").unwrap());
    assert_eq!(view["net"]["code"], "no_nets", "{view}");
    let refused = failure(&calls::draw_net(&state, "").unwrap_err());
    assert_eq!(refused["code"], "no_nets", "{refused}");
}

/// A refusal says why as values, and leaves the session unchanged;
/// `split_passes` greys a split that cannot pass.
#[wasm_bindgen_test(unsupported = test)]
fn a_refused_step_says_why() {
    let started = calls::start("A * B |- B * A", "classical", "").unwrap();
    let state = session(&started);
    let goal = value(&started)["view"]["goals"][0]["goal"]
        .as_u64()
        .unwrap() as u32;
    let refused = calls::apply(&state, goal, &step(0, "ax", None), "").unwrap_err();
    let refused = failure(&refused);
    assert_eq!(
        (refused["code"].clone(), refused["kind"].clone()),
        (json!("step"), json!("invalid"))
    );
    assert!(refused["details"]["kind"].is_string(), "{refused}");
    // ⊢ A⊥ ⅋ B⊥, B ⊗ A: the ⊗ with nothing on its left cannot close.
    let passes = |left| calls::split_passes(&state, goal, &step(1, "⊗", Some(left)), "").unwrap();
    assert!(!passes(vec![]));
}

/// A position past what 32 bits hold is refused, never read as a smaller
/// one.
#[wasm_bindgen_test(unsupported = test)]
fn a_position_past_32_bits_is_refused() {
    let started = calls::start("A |- A", "classical", "").unwrap();
    let state = session(&started);
    let goal = value(&started)["view"]["goals"][0]["goal"]
        .as_u64()
        .unwrap() as u32;
    let wide = json!({"position": 1_u64 << 32, "rule": "ax"}).to_string();
    let refused = failure(&calls::apply(&state, goal, &wide, "").unwrap_err());
    assert!(
        matches!(refused["code"].as_str(), Some("json" | "step")),
        "{refused}"
    );
    assert!(calls::apply(&state, goal, &step(0, "ax", None), "").is_ok());
}

/// Following the hints alone proves a sequent: each hint is the first
/// steps of a proof the search found for the goal, which the page applies
/// in turn, each on the goal the one before opened. A contraction comes
/// with the rule that uses both copies, so the hints do not contract and
/// weaken a formula in turn.
#[wasm_bindgen_test(unsupported = test)]
fn following_the_hints_proves_a_sequent() {
    for (sequent, mode) in [
        ("A, A -o B |- B", "classical"),
        ("!A |- A * A", "intuitionistic"),
        ("!(A -o B), A, A |- B * B", "classical"),
        ("|- A, ~A, B, ~B", "mix"),
    ] {
        let mut state = session(&calls::start(sequent, mode, "").unwrap());
        for _ in 0..20 {
            let view = value(&calls::view(&state, "").unwrap());
            let Some(goal) = view["goals"][0]["goal"].as_u64() else {
                break;
            };
            let hint = value(&calls::hint(&state, goal as u32, "").unwrap());
            let mut goal = goal as u32;
            for step in hint["steps"].as_array().unwrap() {
                let applied = value(&calls::apply(&state, goal, &step.to_string(), "").unwrap());
                state = applied["session"].to_string();
                goal = applied["opened"][0].as_u64().map_or(goal, |g| g as u32);
            }
        }
        assert_eq!(
            value(&calls::view(&state, "").unwrap())["complete"],
            true,
            "{sequent}"
        );
    }
}

/// With `interactive.invertible` on, starting and every step apply the
/// invertible rules, the view names the inferences they closed, a hint
/// says whether its rule is invertible, following the hints still proves
/// a sequent, and `invert_all` applies them to a session started without.
#[wasm_bindgen_test(unsupported = test)]
fn the_invertible_rules_apply_by_the_setting() {
    assert!(!tab::defaults().interactive.invertible);
    let on = settings(&json!({"interactive": {"invertible": true}}));
    let sequent = "A -o B, B -o C |- A -o C";
    let started = value(&calls::start(sequent, "classical", &on).unwrap());
    assert_eq!(started["inverted"], 1);
    assert_eq!(started["view"]["invertible"], json!([0]));
    let goal = started["view"]["goals"][0]["goal"].as_u64().unwrap() as u32;
    let hint = value(&calls::hint(&started["session"].to_string(), goal, &on).unwrap());
    assert_eq!(
        (hint["steps"][0]["rule"].clone(), hint["invertible"].clone()),
        (json!("⊗"), json!(false))
    );
    let state = started["session"].to_string();
    let applied = value(&calls::apply(&state, goal, &step(0, "⊗", Some(vec![2])), &on).unwrap());
    assert_eq!(
        (applied["inverted"].clone(), applied["opened"].clone()),
        (json!(1), json!([3]))
    );
    assert_eq!(applied["view"]["invertible"], json!([0, 2]));
    let off = calls::start(sequent, "classical", "").unwrap();
    let hint = value(&calls::hint(&session(&off), 0, "").unwrap());
    assert_eq!(
        (hint["steps"][0]["rule"].clone(), hint["invertible"].clone()),
        (json!("⅋"), json!(true))
    );
    assert_eq!(value(&off)["inverted"], 0);
    let inverted = value(&calls::invert_all(&session(&off), "").unwrap());
    assert_eq!(inverted["inverted"], 1);
    for (sequent, mode) in [
        ("!(A -o B), A, A |- B * B", "classical"),
        ("A * B, B -o C |- A * C", "intuitionistic"),
    ] {
        let mut state = session(&calls::start(sequent, mode, &on).unwrap());
        for _ in 0..20 {
            let view = value(&calls::view(&state, &on).unwrap());
            let Some(goal) = view["goals"][0]["goal"].as_u64() else {
                break;
            };
            let hint = value(&calls::hint(&state, goal as u32, &on).unwrap());
            assert_eq!(hint["invertible"], false, "{sequent}");
            let mut goal = goal as u32;
            for step in hint["steps"].as_array().unwrap() {
                let applied = value(&calls::apply(&state, goal, &step.to_string(), &on).unwrap());
                state = applied["session"].to_string();
                goal = applied["opened"][0].as_u64().map_or(goal, |g| g as u32);
            }
        }
        assert_eq!(
            value(&calls::view(&state, &on).unwrap())["complete"],
            true,
            "{sequent}"
        );
    }
}

/// Closing a goal stops at the work budget with an honest unknown, and
/// proves it within the tab's.
#[wasm_bindgen_test(unsupported = test)]
fn closing_stops_at_the_work_budget() {
    let started = calls::start("!(a -o a * a), a |- a * a * a * a", "classical", "").unwrap();
    let state = session(&started);
    let goal = value(&started)["view"]["goals"][0]["goal"]
        .as_u64()
        .unwrap() as u32;
    let tight = settings(&json!({"limits": {"work": 1}}));
    let stopped = value(&calls::close(&state, goal, &tight).unwrap());
    assert_eq!(stopped["outcome"]["verdict"], "unknown");
    assert_eq!(
        stopped["outcome"]["reason"]["kind"], "work_limit",
        "{stopped}"
    );
    assert_eq!(stopped["view"]["complete"], false);
    let closed = value(&calls::close(&state, goal, "").unwrap());
    assert_eq!(closed["outcome"]["verdict"], "proved");
    assert_eq!(closed["view"]["complete"], true);
}

/// Closing every goal keeps to one budget for all their searches: with
/// what the first goal's search takes and half of what the second's
/// does, the first is proved and the second is unknown at the work limit.
/// The focused engine searches here, whose work, unlike the Horn engine's
/// few units, outweighs the check of the proof it finds.
#[wasm_bindgen_test(unsupported = test)]
fn closing_all_goals_shares_one_budget() {
    let sequent = "!(a -o a * a), a, !(b -o b * b), b |- (a * a * a * a) * (b * b * b * b)";
    let started = calls::start(sequent, "classical", "").unwrap();
    let view = &value(&started)["view"];
    let root = view["goals"][0]["goal"].as_u64().unwrap() as u32;
    let formulas = view["goals"][0]["formulas"].as_array().unwrap();
    let text = |i: usize| formulas[i]["text"].as_str().unwrap().to_string();
    let tensor = (0..formulas.len())
        .find(|&i| text(i).contains('a') && text(i).contains('b'))
        .unwrap();
    let left = (0..formulas.len())
        .filter(|&i| i != tensor && !text(i).contains('b'))
        .collect();
    let split = calls::apply(&session(&started), root, &step(tensor, "⊗", Some(left)), "").unwrap();
    let state = session(&split);
    let focus = settings(&json!({"search": {"engine": "focus"}}));
    let all = value(&calls::close_all(&state, &focus).unwrap());
    let work = |i: usize| all["results"][i]["work"].as_u64().unwrap();
    assert_eq!(all["view"]["complete"], true);
    let budget = settings(&json!({
        "search": {"engine": "focus"},
        "limits": {"work": work(0) + work(1) / 2},
    }));
    let shared = value(&calls::close_all(&state, &budget).unwrap());
    assert_eq!(
        shared["results"][0]["outcome"]["verdict"], "proved",
        "{shared}"
    );
    let second = &shared["results"][1]["outcome"];
    assert_eq!(
        (second["verdict"].clone(), second["reason"]["kind"].clone()),
        (json!("unknown"), json!("work_limit"))
    );
}

/// A goal or a position the page gives as a number is an integer from 0
/// to `u32::MAX`, refused otherwise rather than taken modulo 2³².
#[wasm_bindgen_test(unsupported = test)]
fn a_number_from_the_page_is_an_index_or_refused() {
    assert_eq!(calls::index(3.0, "a goal"), Ok(3));
    assert_eq!(calls::index(f64::from(u32::MAX), "a goal"), Ok(u32::MAX));
    for number in [2_f64.powi(32) + 3.0, 1.5, -1.0, f64::NAN, f64::INFINITY] {
        assert!(calls::index(number, "a goal").is_err(), "{number}");
    }
}

/// An unprovable sequent is refuted, its refutation checked by its own
/// checker and certified for Rocq.
#[wasm_bindgen_test(unsupported = test)]
fn a_refutation_is_checked_and_certified() {
    let decided = calls::prove("A * A |- A", "classical", "").unwrap();
    let decided = value(&decided);
    assert_eq!(decided["outcome"]["verdict"], "unprovable");
    assert_eq!(decided["refutation_checked"], true);
    let certificate = calls::certificate(&decided["outcome"].to_string(), "").unwrap();
    assert!(certificate.contains("Qed."), "{certificate}");
}

/// The reader tells a typed sequent, a sequent's JSON and a session apart,
/// and refuses a level it does not know by name.
#[wasm_bindgen_test(unsupported = test)]
fn documents_are_read_by_kind() {
    let typed = value(&calls::read("A, A -o B |- B", "").unwrap());
    assert_eq!(
        (
            typed["kind"].clone(),
            typed["fragment"]["intuitionistic"].clone()
        ),
        (json!("sequent"), json!("IMLL"))
    );
    let json = value(&calls::read(&typed["sequent"].to_string(), "").unwrap());
    assert_eq!(
        (json["kind"].clone(), json["text"].clone()),
        (json!("sequent"), typed["text"].clone())
    );
    let started = calls::start("A |- A", "intuitionistic", "").unwrap();
    let session = value(&calls::read(&session(&started), "").unwrap());
    assert_eq!(
        (session["kind"].clone(), session["mode"].clone()),
        (json!("session"), json!("intuitionistic"))
    );
    let mut later = typed["sequent"].clone();
    later["version"] = json!(99);
    let refused = failure(&calls::read(&later.to_string(), "").unwrap_err());
    assert_eq!(refused["code"], "unsupported_version");
    let parse = failure(&calls::read("A *", "").unwrap_err());
    assert_eq!(parse["code"], "parse", "{parse}");
}

/// The drawings render to PNG and PDF from the font the page passes.
#[wasm_bindgen_test(unsupported = test)]
fn drawings_render_with_the_page_s_font() {
    let font = include_bytes!("../../cli/fonts/Euler-Math.otf");
    let started = calls::start("A |- A", "classical", "").unwrap();
    let drawing = calls::draw(&session(&started), "").unwrap();
    let png = calls::render(&drawing, "png", font, "").unwrap();
    assert_eq!(&png[1..4], b"PNG");
    let dated = settings(
        &json!({"styles": {"pdf": {"date": {"year": 2026, "month": 10, "day": 19, "hour": 12, "minute": 0, "second": 0}}}}),
    );
    let pdf = calls::render(&drawing, "pdf", font, &dated).unwrap();
    assert_eq!(&pdf[..5], b"%PDF-");
}

/// The settings choose an export's math font by name: the drawing asks
/// for it and is laid out in its advances, and only its file sets the
/// text of the render.
#[wasm_bindgen_test(unsupported = test)]
fn an_export_is_set_in_the_font_the_settings_name() {
    let euler = include_bytes!("../../cli/fonts/Euler-Math.otf");
    let stix = include_bytes!("../../cli/fonts/STIXTwoMath-Regular.otf");
    let started = calls::start("A ⊗ B |- A ⊗ B", "classical", "").unwrap();
    let chosen = settings(&json!({"styles": {"svg": {"font": "stix-two"}}}));
    let drawing = calls::export(&session(&started), "svg", &chosen).unwrap();
    let default = calls::export(&session(&started), "svg", "").unwrap();
    assert!(
        drawing.contains("font-family=\"'STIX Two Math', serif\""),
        "{drawing}"
    );
    assert!(default.contains("'Euler Math'"));
    let unnamed = |svg: &str, family: &str| svg.replace(family, "");
    assert_ne!(
        unnamed(&drawing, "'STIX Two Math', serif"),
        unnamed(&default, "'Euler Math', 'Neo Euler', serif"),
    );
    let set = calls::render(&drawing, "png", stix, "").unwrap();
    let unset = calls::render(&drawing, "png", euler, "").unwrap();
    assert_ne!(set, unset, "the drawing's text needs its font's file");
}

/// A search that recurses to the tab's recursion limit stops there with
/// that reason, on each of the paths whose levels take the most stack:
/// copies of a `!` formula (a focus level each), nested `&` (the additive
/// path's level a pair) and a Mix of pairs (a level a part). The module's
/// stack and the host's hold the deepest recursion the limit allows.
#[wasm_bindgen_test(unsupported = test)]
fn a_search_at_the_recursion_limit_stops_there() {
    let depth = tab::RECURSION_DEPTH as usize;
    let nested = (0..2 * depth).fold("a".to_string(), |f, _| format!("(a & {f})"));
    let pairs = vec!["a, ~a"; 2 * depth].join(", ");
    for (sequent, mode) in [
        ("!(a -o a * a), a |- ?b".to_string(), "classical"),
        (format!("|- {nested}, ~a"), "classical"),
        (format!("|- {pairs}"), "mix"),
    ] {
        // The classical refuter, which refutes the first, is off.
        let deep = settings(&json!({
            "search": {"copies": 4 * depth, "classical_work": 0},
            "limits": {"work": null},
        }));
        let decided = value(&on_a_search_stack(move || {
            calls::prove(&sequent, mode, &deep).unwrap()
        }));
        let reason = &decided["outcome"]["reason"];
        assert_eq!(reason["kind"], "recursion_limit", "{decided}");
        assert_eq!(reason["depth"], tab::RECURSION_DEPTH);
    }
}

/// Runs `f` where a search to the tab's recursion limit runs: on the
/// module's own stack under WebAssembly, and elsewhere on a thread with
/// the stack the library asks for that depth, as the command gives its
/// search one (a test thread's is smaller).
fn on_a_search_stack(f: impl FnOnce() -> String + Send + 'static) -> String {
    if cfg!(target_family = "wasm") {
        return f();
    }
    let limits = linlog::Limits::default().with_recursion_depth(tab::RECURSION_DEPTH);
    std::thread::Builder::new()
        .stack_size(limits.stack_bytes())
        .spawn(f)
        .unwrap()
        .join()
        .unwrap()
}

/// A memory bound smaller than a search needs is a refusal or an unknown,
/// never a crash.
#[wasm_bindgen_test(unsupported = test)]
fn a_small_memory_bound_refuses() {
    let small = settings(&json!({
        "limits": {"memory_bytes": 4096, "work": null},
        "search": {"copies": 64, "classical_work": 0},
    }));
    let answer = calls::prove("!(a -o a * a), a |- ?b", "classical", &small);
    match answer {
        Ok(decided) => assert_eq!(value(&decided)["outcome"]["verdict"], "unknown"),
        Err(refused) => assert_eq!(failure(&refused)["kind"], "limit"),
    }
}

/// A goal is filled from another finished session of an equal sequent,
/// its formulas in another order, here in intuitionistic mode; a source
/// of other formulas is refused with them on each side, one not finished
/// is refused, and a proof the goal's mode does not allow is the checker's
/// to refuse.
#[wasm_bindgen_test(unsupported = test)]
fn a_goal_is_filled_from_another_session() {
    let finished = |sequent: &str, mode: &str| {
        let started = calls::start(sequent, mode, "").unwrap();
        session(&calls::close(&session(&started), 0, "").unwrap())
    };
    let target = session(&calls::start("A, A -o B |- B", "intuitionistic", "").unwrap());
    let source = finished("A -o B, A |- B", "intuitionistic");
    let filled = calls::fill(&target, 0, &source, "").unwrap();
    assert_eq!(value(&filled)["view"]["complete"], true, "{filled}");

    let other = finished("B -o A, B |- A", "intuitionistic");
    let refused = failure(&calls::fill(&target, 0, &other, "").unwrap_err());
    assert_eq!(refused["code"], "sequent_mismatch", "{refused}");
    // Every formula of the one differs from every formula of the other.
    let side = |key: &str| refused["details"][key].as_array().unwrap().len();
    assert_eq!((side("proof"), side("goal")), (3, 3), "{refused}");
    let open = session(&calls::start("A |- A", "classical", "").unwrap());
    let refused = failure(&calls::fill(&target, 0, &open, "").unwrap_err());
    assert_eq!(refused["code"], "open_goals", "{refused}");

    let classical = session(&calls::start("|- A, ~A, B, ~B", "classical", "").unwrap());
    let mixed = finished("|- B, ~B, A, ~A", "mix");
    let refused = failure(&calls::fill(&classical, 0, &mixed, "").unwrap_err());
    assert_eq!(refused["code"], "invalid_proof", "{refused}");
}

/// In intuitionistic mode each goal formula carries its reading, which
/// its text is too: a hypothesis `A ⊸ B` reads `A ⊸ B` (the one-sided
/// sequent holds `A ⊗ ~B`), and so does the formula under a `!` that a
/// promotion opened as a goal; in classical mode there is no reading.
#[wasm_bindgen_test(unsupported = test)]
fn a_goal_s_formulas_carry_their_reading() {
    let formulas = |answer: &str| value(answer)["view"]["goals"][0]["formulas"].clone();
    let readings = |answer: &str| {
        let mut all: Vec<String> = formulas(answer)
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["reading"].as_str().unwrap().to_owned())
            .collect();
        all.sort();
        all
    };
    let started = calls::start("A, A -o B |- B", "intuitionistic", "").unwrap();
    assert_eq!(readings(&started), ["A", "A ⊸ B", "B"], "{started}");
    let text = formulas(&started)
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["text"] == "A ⊸ B");
    assert!(text, "{started}");

    let banged = calls::start("!(A -o B) |- !(A -o B)", "intuitionistic", "").unwrap();
    let output = formulas(&banged)
        .as_array()
        .unwrap()
        .iter()
        .position(|f| f["side"] == "output")
        .unwrap();
    let promoted = calls::apply(&session(&banged), 0, &step(output, "!", None), "").unwrap();
    assert_eq!(readings(&promoted), ["!(A ⊸ B)", "A ⊸ B"], "{promoted}");

    let classical = calls::start("A, A -o B |- B", "classical", "").unwrap();
    let all = formulas(&classical);
    assert!(
        all.as_array()
            .unwrap()
            .iter()
            .all(|f| f.get("reading").is_none()),
        "{classical}"
    );
}

/// A random formula of intuitionistic linear logic over `a` and `b`, at
/// most `depth` connectives deep, from the state of a linear congruence.
fn ill_formula(state: &mut u64, depth: u32) -> String {
    let mut next = |n: u64| {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (*state >> 33) % n
    };
    if depth == 0 || next(3) == 0 {
        return ["a", "b", "a", "b", "1", "top", "0"][next(7) as usize].to_owned();
    }
    let connective = next(6);
    let left = ill_formula(state, depth - 1);
    if connective == 5 {
        return format!("!{left}");
    }
    let right = ill_formula(state, depth - 1);
    let op = ["*", "-o", "&", "+", "-o"][connective as usize];
    format!("({left} {op} {right})")
}

/// No text the bindings write of an intuitionistic sequent has a symbol
/// of classical linear logic: over random sequents, the sequent `read_as`
/// writes, every goal formula's text and reading, and every export of the
/// session once all its goals are closed (of a Rocq certificate the
/// comments, whose code is Rocq's, with no classical constructor), the
/// net drawn, and every step of the elimination of the session's cut; a
/// sequent written classically is refused with a hint.
#[wasm_bindgen_test(unsupported = test)]
fn no_intuitionistic_text_writes_a_classical_symbol() {
    const CLASSICAL: [&str; 8] = ["~", "⅋", "⊥", "?", r"\parr", r"\bot", r"\wn", "perp"];
    let mut state = 41;
    let (mut finished, mut nets) = (0, 0);
    for _ in 0..16 {
        let hypotheses: Vec<String> = (0..state % 3).map(|_| ill_formula(&mut state, 2)).collect();
        let goal = ill_formula(&mut state, 3);
        let sequent = format!("{} |- {goal}", hypotheses.join(", "));
        let check = |what: &str, out: &str| {
            let out = out.replace("<?xml", "").replace("?>", "");
            let rocq = what == "rocq";
            let comments: String = out
                .lines()
                .filter(|line| !rocq || line.starts_with("(*"))
                .collect::<Vec<_>>()
                .join("\n");
            for symbol in CLASSICAL {
                assert!(
                    !comments.contains(symbol),
                    "{sequent:?}: {what} writes {symbol:?}:\n{out}"
                );
            }
            for word in ["parr", "wn ", "dual", " ll "] {
                assert!(
                    !rocq || !out.contains(word),
                    "{sequent:?}: {what} writes {word:?}"
                );
            }
        };
        let read = value(&calls::read_as(&sequent, "intuitionistic", "").unwrap());
        check("read_as", read["text"].as_str().unwrap());
        let started = calls::start(&sequent, "intuitionistic", "").unwrap();
        // Cut on the goal, typed as written: the hypotheses prove it, and
        // it proves itself.
        let root = value(&started)["view"]["goals"][0]["goal"]
            .as_u64()
            .unwrap() as u32;
        let left = json!((0..hypotheses.len()).collect::<Vec<_>>()).to_string();
        let cut = calls::cut(&session(&started), root, &goal, &left, "").unwrap();
        let closed = calls::close_all(&session(&cut), "").unwrap();
        for answer in [&started, &cut, &closed] {
            for goal in value(answer)["view"]["goals"].as_array().unwrap() {
                for formula in goal["formulas"].as_array().unwrap() {
                    check("a goal's text", formula["text"].as_str().unwrap());
                    check("a goal's reading", formula["reading"].as_str().unwrap());
                }
            }
        }
        if value(&closed)["view"]["complete"] != true {
            continue;
        }
        finished += 1;
        for format in ["text", "latex", "typst", "svg", "rocq"] {
            check(
                format,
                &calls::export(&session(&closed), format, "").unwrap(),
            );
        }
        if let Ok(net) = calls::draw_net(&session(&closed), "") {
            check("the net", &net);
        }
        // Every step of the cut's elimination, on the derivation and on
        // the net, each drawn in the intuitionistic syntax.
        let ran = value(&calls::eliminate(&session(&closed), "", "").unwrap());
        for step in ran["steps"].as_array().unwrap() {
            check("a step's formula", step["formula"].as_str().unwrap());
            check("a step's drawing", step["drawing"].as_str().unwrap());
        }
        check("the proof reached", ran["drawing"].as_str().unwrap());
        if let Ok(ran) = calls::net_eliminate(&session(&closed), "", "") {
            let ran = value(&ran);
            for step in ran["steps"].as_array().unwrap() {
                check("a net step's formula", step["formula"].as_str().unwrap());
                check("a net step's drawing", step["drawing"].as_str().unwrap());
            }
            check("the net reached", ran["drawing"].as_str().unwrap());
            nets += 1;
        }
    }
    assert!(finished > 3, "{finished} of 16 finished");
    // Sequents with nets, which the random ones, with units and additives,
    // seldom have.
    for (sequent, formula, left) in [
        ("a, a -o b |- b", "a -o b", "[1]"),
        ("a * b |- b * a", "b * a", "[0]"),
        ("!a |- a * a", "!a", "[0]"),
    ] {
        let closed = cut_session(sequent, "intuitionistic", formula, left);
        let ran = value(&calls::net_eliminate(&closed, "", "").unwrap());
        for step in ran["steps"].as_array().unwrap() {
            for key in ["formula", "drawing"] {
                let text = step[key]
                    .as_str()
                    .unwrap()
                    .replace("<?xml", "")
                    .replace("?>", "");
                for symbol in CLASSICAL {
                    assert!(
                        !text.contains(symbol),
                        "{sequent:?}: a net step's {key}: {text}"
                    );
                }
            }
        }
        nets += 1;
    }
    assert!(nets >= 3, "{nets} nets");
    let refused = failure(&calls::read_as("~a |- b", "intuitionistic", "").unwrap_err());
    assert!(
        refused["message"].as_str().unwrap().contains("no negation"),
        "{refused}"
    );
}

/// Returns the finished session of `sequent` in `mode` cut on `formula`,
/// the members at `left` going to its first premise, both premises closed
/// by the search.
fn cut_session(sequent: &str, mode: &str, formula: &str, left: &str) -> String {
    let started = calls::start(sequent, mode, "").unwrap();
    let root = value(&started)["view"]["goals"][0]["goal"]
        .as_u64()
        .unwrap() as u32;
    let cut = calls::cut(&session(&started), root, formula, left, "").unwrap();
    let closed = calls::close_all(&session(&cut), "").unwrap();
    assert_eq!(value(&closed)["view"]["complete"], true, "{sequent}");
    session(&closed)
}

/// Returns the strings of a JSON array.
fn strings(array: &Value) -> Vec<String> {
    array
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_owned())
        .collect()
}

/// A proof with a cut is eliminated step by step, on the derivation drawn
/// as the session is (two-sided as written, and intuitionistic): its cut
/// listed with its formula and the drawing's inferences that show it, the
/// cut named and the strategy's next one taking the same step, each step
/// named by its case with the ids of the cut in the drawing of the proof
/// it was taken on, until no cut is left; `eliminate` takes the same steps
/// to the same proof, every step drawn with its cut; a node that is no cut
/// and a proof of another sequent are refused.
#[wasm_bindgen_test(unsupported = test)]
fn a_proof_with_cuts_is_eliminated_step_by_step() {
    let mut proofs = Vec::new();
    for (sequent, mode, formula, left, written) in [
        ("A, A -o B |- B", "classical", "A -o B", "[1]", "~A ⅋ B"),
        ("!a |- a * a", "intuitionistic", "!a", "[0]", "!a"),
    ] {
        let closed = cut_session(sequent, mode, formula, left);
        let two = settings(&json!({"view": {"classical": "two"}}));
        let view = value(&calls::cuts(&closed, "", &two).unwrap());
        let drawing = view["drawing"].as_str().unwrap();
        let cuts = view["cuts"].as_array().unwrap();
        assert_eq!(cuts.len(), 1, "{sequent}");
        assert_eq!(cuts[0]["formula"], written);
        assert_eq!(view["next"], cuts[0]["node"]);
        let shown: Vec<String> = (cuts[0]["inferences"].as_array().unwrap().iter())
            .map(|n| format!("i{n}"))
            .collect();
        assert_ne!(shown, Vec::<String>::new());
        for id in &shown {
            assert!(drawing.contains(&format!("id=\"{id}\"")), "{id}: {drawing}");
        }
        let node = cuts[0]["node"].as_u64().unwrap() as u32;
        let named = value(&calls::cut_step(&closed, "", Some(node), &two).unwrap());
        let next = value(&calls::cut_step(&closed, "", None, &two).unwrap());
        assert_eq!(named, next);
        assert_eq!(strings(&named["step"]["highlight"]["cut"]), shown);
        assert_eq!(named["step"]["formula"], written);
        let mut cases = vec![named["step"]["case"].clone()];
        let mut current = named;
        while !current["next"].is_null() {
            let before = current["drawing"].as_str().unwrap().to_owned();
            let proof = current["proof"].to_string();
            current = value(&calls::cut_step(&closed, &proof, None, &two).unwrap());
            let step = &current["step"];
            assert!(step["name"].as_str().is_some_and(|name| !name.is_empty()));
            for id in strings(&step["highlight"]["cut"]) {
                assert!(before.contains(&format!("id=\"{id}\"")), "{id}");
            }
            cases.push(step["case"].clone());
            assert!(cases.len() < 100, "{sequent}: {cases:?}");
        }
        assert_eq!(current["cuts"], json!([]));
        let ran = value(&calls::eliminate(&closed, "", &two).unwrap());
        assert_eq!(
            (&ran["end"], &ran["undrawn"]),
            (&json!("cut_free"), &Value::Null)
        );
        assert_eq!(ran["proof"], current["proof"]);
        let steps = ran["steps"].as_array().unwrap();
        assert_eq!(
            steps.iter().map(|s| s["case"].clone()).collect::<Vec<_>>(),
            cases
        );
        for step in steps {
            let drawing = step["drawing"].as_str().unwrap();
            for id in strings(&step["highlight"]["cut"]) {
                assert!(drawing.contains(&format!("id=\"{id}\"")), "{id}");
            }
        }
        let axiom = (current["proof"]["nodes"].as_array().unwrap().iter())
            .position(|n| n.get("ax").is_some())
            .unwrap() as u32;
        let refused = failure(&calls::cut_step(&closed, "", Some(axiom), "").unwrap_err());
        assert_eq!(refused["code"], "no_cut");
        proofs.push((closed, current["proof"].to_string()));
    }
    let refused = failure(&calls::cuts(&proofs[0].0, &proofs[1].1, "").unwrap_err());
    assert_eq!(refused["code"], "foreign_proof");
}

/// An elimination to the end stops at `cut.steps` with the proof reached
/// and its cuts, saying which bound; past `cut.drawn_bytes` its steps are
/// taken undrawn, from the first whose drawing would pass it; the tab's
/// bound on the drawings is its own.
#[wasm_bindgen_test(unsupported = test)]
fn an_elimination_stops_at_its_bounds() {
    assert_eq!(tab::defaults().cut.drawn_bytes, Some(tab::DRAWN_BYTES));
    let closed = cut_session("!a |- a * a", "intuitionistic", "!a", "[0]");
    let one = settings(&json!({"cut": {"steps": 1}}));
    let ran = value(&calls::eliminate(&closed, "", &one).unwrap());
    assert_eq!(ran["end"], "refused");
    assert_eq!(ran["reason"]["code"], "elimination_limit");
    assert_eq!(ran["steps"].as_array().unwrap().len(), 1);
    assert_ne!(ran["cuts"], json!([]));
    let small = settings(&json!({"cut": {"drawn_bytes": 1}}));
    let ran = value(&calls::eliminate(&closed, "", &small).unwrap());
    assert_eq!(
        (&ran["end"], &ran["undrawn"]),
        (&json!("cut_free"), &json!(1))
    );
    for step in ran["steps"].as_array().unwrap() {
        assert_eq!(
            (&step["drawing"], &step["highlight"]),
            (&Value::Null, &Value::Null)
        );
    }
    let ran = value(&calls::net_eliminate(&closed, "", &small).unwrap());
    assert_eq!(
        (&ran["end"], &ran["undrawn"]),
        (&json!("cut_free"), &json!(1))
    );
}

/// A proof with a cut has a net with a cut link, eliminated step by step:
/// the link listed with the id of its bar in the drawing, either end or
/// the strategy's next one taking the same step, the net it was taken on
/// drawn with its marks, each step named by its case; `net_eliminate`
/// takes the same steps to the same net. With exponentials the net has
/// boxes, and what a step acted on besides the cut is an element of its
/// drawing; a proof that cuts one occurrence twice has a net with boxes
/// too. A vertex at no cut link is refused.
#[wasm_bindgen_test(unsupported = test)]
fn a_net_with_cuts_is_eliminated_step_by_step() {
    let mll = cut_session("A, A -o B |- B", "classical", "A -o B", "[1]");
    let view = value(&calls::net_cuts(&mll, "", "").unwrap());
    assert_eq!(view["boxes"], false);
    let link = &view["cuts"][0];
    let id = link["id"].as_str().unwrap();
    assert!(
        view["drawing"]
            .as_str()
            .unwrap()
            .contains(&format!("id=\"{id}\""))
    );
    let ends: Vec<u32> = (link["ends"].as_array().unwrap().iter())
        .map(|v| v.as_u64().unwrap() as u32)
        .collect();
    let first = value(&calls::net_cut_step(&mll, "", Some(ends[0]), "").unwrap());
    let second = value(&calls::net_cut_step(&mll, "", Some(ends[1]), "").unwrap());
    let next = value(&calls::net_cut_step(&mll, "", None, "").unwrap());
    assert_eq!((&first, &first), (&second, &next));
    assert_eq!(strings(&first["step"]["highlight"]["cut"]), [id]);
    assert!(
        first["marked"]
            .as_str()
            .unwrap()
            .contains(&format!("id=\"{id}\""))
    );
    let mut current = first;
    let mut cases = vec![current["step"]["case"].clone()];
    while !current["next"].is_null() {
        let net = current["net"].to_string();
        current = value(&calls::net_cut_step(&mll, &net, None, "").unwrap());
        cases.push(current["step"]["case"].clone());
        assert!(cases.len() < 100, "{cases:?}");
    }
    let ran = value(&calls::net_eliminate(&mll, "", "").unwrap());
    assert_eq!(ran["end"], "cut_free");
    assert_eq!(ran["net"], current["net"]);
    let ran_cases: Vec<Value> = (ran["steps"].as_array().unwrap().iter())
        .map(|s| s["case"].clone())
        .collect();
    assert_eq!(ran_cases, cases);
    let conclusion = view["net"]["links"][0][0].as_u64().unwrap() as u32;
    let refused = failure(&calls::net_cut_step(&mll, "", Some(conclusion), "").unwrap_err());
    assert_eq!(refused["code"], "no_cut");

    let mell = cut_session("!a |- a * a", "intuitionistic", "!a", "[0]");
    let ran = value(&calls::net_eliminate(&mell, "", "").unwrap());
    assert_eq!(
        (&ran["boxes"], &ran["end"]),
        (&json!(true), &json!("cut_free"))
    );
    let mut acted = 0;
    for step in ran["steps"].as_array().unwrap() {
        let drawing = step["drawing"].as_str().unwrap();
        for id in strings(&step["highlight"]["cut"])
            .into_iter()
            .chain(strings(&step["highlight"]["above"]))
        {
            assert!(drawing.contains(&format!("id=\"{id}\"")), "{id}: {drawing}");
        }
        acted += step["highlight"]["above"].as_array().unwrap().len();
    }
    assert!(acted > 0, "{ran}");

    // ⊢ a ⊗ a, ~a, ~a: 0 ⊗, 1 a, 2 a, 3 ~a, 4 ~a; the cut formula a, 5,
    // and its dual, 6, cut under each premise of the ⊗.
    let twice = cut_session("|- a * a, ~a, ~a", "classical", "a", "[1]");
    let mut proof = value(&calls::proof(&twice, "").unwrap());
    proof["nodes"] = json!([
        {"ax": [5, 3]}, {"ax": [1, 6]}, {"cut": [5, 0, 1]},
        {"ax": [5, 4]}, {"ax": [2, 6]}, {"cut": [5, 3, 4]}, {"⊗": [0, 2, 5]}
    ]);
    let view = value(&calls::net_cuts(&twice, &proof.to_string(), "").unwrap());
    assert_eq!(view["boxes"], true);
    assert_eq!(view["cuts"].as_array().unwrap().len(), 2);
    let ran = value(&calls::net_eliminate(&twice, &proof.to_string(), "").unwrap());
    assert_eq!(ran["end"], "cut_free");
}

/// A first-order session through the calls: a sequent with quantifiers and
/// terms read and written back, the four quantifier rules offered (a
/// witness taken by `∃R` and `∀L` alone), `∃L` and `∀R` making
/// eigenvariables, `∀L` with its witness left open as `$1` and `∃R` with
/// one given as text, the axiom filling the open one, the finished proof
/// checked, drawn and exported; a witness that does not read and the
/// eigenvariable condition are refused in words.
#[wasm_bindgen_test(unsupported = test)]
fn a_first_order_session_takes_the_quantifier_rules() {
    let text = "exists y. forall x. r(x, y) |- forall x. exists y. r(x, y)";
    let read = value(&calls::read_as(text, "intuitionistic", "").unwrap());
    assert_eq!(read["text"], "∃y. (∀x. r(x, y)) ⊢ ∀x. (∃y. r(x, y))");
    let answer = calls::start(text, "intuitionistic", "").unwrap();
    assert_eq!(value(&answer)["view"]["fragment"], "IMLL1");
    let root = open_goals(&answer)[0];
    let state = session(&answer);
    let hypothesis = at(&answer, root, "∃y. (∀x. r(x, y))");
    assert_eq!(
        value(&calls::rules(&state, root, hypothesis as u32, "").unwrap()),
        json!([{"rule": "∃L", "split": false}])
    );
    let answer = calls::apply(&state, root, &witnessed(hypothesis, "∃L", None), "").unwrap();
    let goal = open_goals(&answer)[0];
    let answer = calls::apply(
        &session(&answer),
        goal,
        &witnessed(at(&answer, goal, "∀x. (∃y. r(x, y))"), "∀R", None),
        "",
    )
    .unwrap();
    let goal = open_goals(&answer)[0];
    let state = session(&answer);
    let position = at(&answer, goal, "∀x. r(x, a)");
    assert_eq!(
        value(&calls::rules(&state, goal, position as u32, "").unwrap()),
        json!([{"rule": "∀L", "split": false, "witness": true}])
    );
    let error =
        calls::apply(&state, goal, &witnessed(position, "∀L", Some("g(a)")), "").unwrap_err();
    assert_eq!(failure(&error)["details"]["kind"], "witness", "{}", error.0);
    let answer = calls::apply(&state, goal, &witnessed(position, "∀L", None), "").unwrap();
    let goal = open_goals(&answer)[0];
    at(&answer, goal, "r($1, a)");
    let answer = calls::apply(
        &session(&answer),
        goal,
        &witnessed(at(&answer, goal, "∃y. r(b, y)"), "∃R", Some("a")),
        "",
    )
    .unwrap();
    let goal = open_goals(&answer)[0];
    at(&answer, goal, "r(b, a)");
    let answer = calls::apply(&session(&answer), goal, &step(0, "ax", None), "").unwrap();
    let state = session(&answer);
    assert_eq!(value(&answer)["view"]["complete"], true);
    // Each quantifier step with its term as taken, the open one `null`
    // though the axiom filled it.
    let terms: Vec<Value> = value(&answer)["view"]["opened"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["term"].clone())
        .collect();
    assert_eq!(terms, [json!("a"), json!("b"), Value::Null, json!("a")]);
    // A member past the forest reads its occurrence off the instances.
    let (view, doc) = (&value(&answer)["view"], value(&state));
    let forest = view["forest"].as_u64().unwrap();
    let leaf = doc["inferences"].as_array().unwrap().last().unwrap()["sequent"][1]
        .as_u64()
        .unwrap();
    assert!(leaf >= forest, "{doc}");
    let occurrence = &doc["instances"]["members"][(leaf - forest) as usize][0];
    assert!(occurrence.as_u64().unwrap() < forest, "{doc}");
    assert_eq!(value(&calls::proof(&state, "").unwrap())["version"], 7);
    // No text of the intuitionistic session is one-sided.
    let shown = calls::draw(&state, "").unwrap();
    assert!(!shown.contains('⅋') && !shown.contains('~'), "{shown}");
    let drawing = calls::draw(&state, "").unwrap();
    assert!(
        drawing.contains("∀L") && drawing.contains("r(b, a)"),
        "{drawing}"
    );
    let latex = calls::export(&state, "latex", "").unwrap();
    assert!(latex.contains("forall"), "{latex}");

    // `$1` is made before `b`, which it would name.
    let text = "forall x. exists y. r(x, y) |- exists y. forall x. r(x, y)";
    let answer = calls::start(text, "intuitionistic", "").unwrap();
    let mut answer = answer;
    for (formula, rule, witness) in [
        ("∃y. (∀x. r(x, y))", "∃R", None),
        ("∀x. r(x, $1)", "∀R", None),
        ("∀x. (∃y. r(x, y))", "∀L", Some("a")),
        ("∃y. r(a, y)", "∃L", None),
    ] {
        let goal = open_goals(&answer)[0];
        let position = at(&answer, goal, formula);
        answer = calls::apply(
            &session(&answer),
            goal,
            &witnessed(position, rule, witness),
            "",
        )
        .unwrap();
    }
    let goal = open_goals(&answer)[0];
    let error = calls::apply(&session(&answer), goal, &step(0, "ax", None), "").unwrap_err();
    let error = failure(&error);
    assert_eq!(error["details"]["kind"], "eigenvariable", "{error}");
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("eigenvariable b cannot stand there"),
        "{error}"
    );
}

/// Undo after the session went through its JSON, as every call takes it,
/// gives the eigenvariable letters and the table of a session that never
/// left memory.
#[wasm_bindgen_test(unsupported = test)]
fn undo_through_the_calls_keeps_a_live_session_s_letters() {
    use linlog::{Interactive, Mode, Rule, Sequent, Step};
    let text = "|- forall x. exists y. p(f(x), y) -o p(f(x), y)";
    let sequent: Sequent = text.parse().unwrap();
    let mut live = Interactive::new(&sequent, Mode::CLASSICAL).unwrap();
    let mut answer = calls::start(text, "classical", "").unwrap();
    for round in 0..3 {
        let goal = open_goals(&answer)[0];
        answer = calls::apply(&session(&answer), goal, &witnessed(0, "∀", None), "").unwrap();
        let g = live.goals().next().unwrap();
        live.apply(g, &Step::new(0, Rule::Forall)).unwrap();
        let goal = open_goals(&answer)[0];
        at(&answer, goal, "∃y. (~p(f(a), y) ⅋ p(f(a), y))");
        answer = calls::apply(
            &session(&answer),
            goal,
            &witnessed(0, "∃", Some("f(a)")),
            "",
        )
        .unwrap();
        let g = live.goals().next().unwrap();
        let w = live.witness("f(a)").unwrap();
        live.apply(g, &Step::new(0, Rule::Exists).with_witness(w))
            .unwrap();
        for _ in 0..2 {
            answer = calls::undo(&session(&answer), "").unwrap();
            live.undo();
        }
        let (page, live) = (
            value(&session(&answer)),
            serde_json::to_value(&live).unwrap(),
        );
        for key in ["inferences", "history", "instances"] {
            assert_eq!(page[key], live[key], "round {round}");
        }
    }
    let goal = open_goals(&answer)[0];
    answer = calls::apply(&session(&answer), goal, &witnessed(0, "∀", None), "").unwrap();
    at(
        &answer,
        open_goals(&answer)[0],
        "∃y. (~p(f(a), y) ⅋ p(f(a), y))",
    );
}

/// A hint on a goal of instances: the goal closed in a copy of the
/// session, its first steps read from it, an `∃R` with the witness the
/// search chose; following the hints proves the sequent.
#[wasm_bindgen_test(unsupported = test)]
fn following_first_order_hints_proves_a_sequent() {
    let text = "forall x. (p(x) -o q(x)), forall x. p(x) |- forall y. q(y)";
    let mut answer = calls::start(text, "intuitionistic", "").unwrap();
    let mut witnesses = Vec::new();
    for _ in 0..20 {
        let Some(&goal) = open_goals(&answer).first() else {
            break;
        };
        let state = session(&answer);
        let hint = value(&calls::hint(&state, goal, "").unwrap());
        assert_eq!(hint["outcome"]["verdict"], "proved", "{hint}");
        let steps = hint["steps"].as_array().unwrap().clone();
        let mut goal = goal;
        let mut state = state;
        for step in &steps {
            if let Some(w) = step["witness"].as_str() {
                witnesses.push(w.to_owned());
            }
            answer = calls::apply(&state, goal, &step.to_string(), "").unwrap();
            state = session(&answer);
            if let Some(next) = value(&answer)["opened"][0].as_u64() {
                goal = next as u32;
            }
        }
    }
    assert_eq!(value(&answer)["view"]["complete"], true, "{answer}");
    assert!(witnesses.iter().any(|w| w == "a"), "{witnesses:?}");
    calls::proof(&session(&answer), "").unwrap();
}

/// A cut at first order through the calls: on `p(a)` under the `∀R` that
/// made `a`, its view's formula as typed, the finished proof checked and
/// its cut eliminated step by step to the end; an eigenvariable of another
/// branch refused in words.
#[wasm_bindgen_test(unsupported = test)]
fn a_first_order_cut_is_made_and_eliminated() {
    let text = "forall x. (p(x) -o q(x)), forall x. p(x) |- forall y. q(y)";
    let answer = calls::start(text, "intuitionistic", "").unwrap();
    let root = open_goals(&answer)[0];
    let answer = calls::apply(
        &session(&answer),
        root,
        &witnessed(at(&answer, root, "∀y. q(y)"), "∀R", None),
        "",
    )
    .unwrap();
    let goal = open_goals(&answer)[0];
    let left = at(&answer, goal, "∀x. p(x)");
    let answer = calls::cut(&session(&answer), goal, "p(a)", &format!("[{left}]"), "").unwrap();
    assert_eq!(
        value(&answer)["view"]["cuts"][0]["formula"],
        "p(a)",
        "{answer}"
    );
    let error =
        calls::cut(&session(&answer), open_goals(&answer)[0], "r(a)", "[]", "").unwrap_err();
    assert_eq!(
        failure(&error)["details"]["kind"],
        "cut_symbol",
        "{}",
        error.0
    );
    let answer = calls::close_all(&session(&answer), "").unwrap();
    let state = session(&answer);
    assert_eq!(value(&answer)["view"]["complete"], true, "{answer}");
    calls::proof(&state, "").unwrap();
    assert!(calls::draw(&state, "").unwrap().contains("cut"));
    let cuts = value(&calls::cuts(&state, "", "").unwrap());
    assert_eq!(cuts["cuts"][0]["formula"], "p(a)", "{cuts}");
    let end = value(&calls::eliminate(&state, "", "").unwrap());
    assert_eq!(end["end"], "cut_free", "{end}");
    assert_ne!(end["steps"], json!([]), "{end}");

    let text = "forall x. p(x), forall x. q(x) |- (forall y. p(y)) * (forall y. q(y))";
    let answer = calls::start(text, "intuitionistic", "").unwrap();
    let root = open_goals(&answer)[0];
    let tensor = at(&answer, root, "(∀y. p(y)) ⊗ (∀y. q(y))");
    let left = at(&answer, root, "∀x. p(x)");
    let answer = calls::apply(
        &session(&answer),
        root,
        &step(tensor, "⊗R", Some(vec![left])),
        "",
    )
    .unwrap();
    let [first, second] = open_goals(&answer)[..] else {
        panic!("two goals");
    };
    let answer = calls::apply(
        &session(&answer),
        first,
        &witnessed(at(&answer, first, "∀y. p(y)"), "∀R", None),
        "",
    )
    .unwrap();
    let error = calls::cut(&session(&answer), second, "q(a)", "[0]", "").unwrap_err();
    let error = failure(&error);
    assert_eq!(error["details"]["kind"], "eigenvariable", "{error}");
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("eigenvariable a"),
        "{error}"
    );
}

/// The first-order bounds are the tab's settings: the term limit the
/// tab's occurrences, read over by the page's `limits.terms`, which
/// refuses a witness past it; `search.witness_depth` stops a search
/// that needs deeper witnesses; `search.instances` is read.
#[wasm_bindgen_test(unsupported = test)]
fn the_first_order_bounds_are_settings() {
    let defaults = value(&calls::defaults());
    assert_eq!(defaults["limits"]["terms"], tab::TERMS);
    assert!(defaults["search"]["witness_depth"].is_u64(), "{defaults}");
    let text = "forall x. p(x) |- p(f(f(f(a))))";
    let answer = calls::start(text, "intuitionistic", "").unwrap();
    let root = open_goals(&answer)[0];
    let small = settings(&json!({"limits": {"terms": 3}}));
    let error = calls::apply(
        &session(&answer),
        root,
        &witnessed(0, "∀L", Some("f(f(f(f(a))))")),
        &small,
    )
    .unwrap_err();
    assert_eq!(failure(&error)["details"]["kind"], "witness", "{}", error.0);
    let shallow = settings(&json!({"search": {"witness_depth": 1, "instances": 8}}));
    let closed = value(&calls::close(&session(&answer), root, &shallow).unwrap());
    assert_eq!(closed["outcome"]["verdict"], "unknown", "{closed}");
    let closed = value(&calls::close(&session(&answer), root, "").unwrap());
    assert_eq!(closed["outcome"]["verdict"], "proved", "{closed}");
}
