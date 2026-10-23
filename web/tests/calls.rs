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
    })
    .unwrap()
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

/// In intuitionistic mode each goal formula carries its reading beside its
/// one-sided text: a hypothesis `A ⊸ B` reads `A ⊸ B`, and so does the
/// formula under a `!` that a promotion opened as a goal; in classical
/// mode there is no reading.
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
        .any(|f| f["text"] == "A ⊗ ~B");
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
