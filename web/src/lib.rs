// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! linlog for a web page: the library compiled to WebAssembly behind
//! calls of JSON text in and JSON or SVG text out, over the interactive
//! state, the settings and a search's outcome.
//!
//! Every call takes the documents it works on and the page's settings
//! (the JSON of [`linlog::Settings`], read over the tab's defaults: an
//! empty text is the defaults, a key replaces the default there) and
//! returns a document; nothing is kept between calls. A call that fails
//! throws the error's JSON as a string, `{"code", "kind", "message",
//! "setting", "details"}`, which the page words itself. The module reads
//! no clock and starts no thread: a call ends at the bounds of the
//! settings, the work bound first, or when the page stops its worker.
//!
//! The calls are those of [`calls`], each exported under its name.

pub mod calls;
mod failure;
pub mod tab;

pub use failure::Failure;

/// The exports of the module, which wrap [`calls`] for JavaScript.
mod exports {
    use crate::calls;
    use crate::failure::Failure;
    use wasm_bindgen::prelude::{JsValue, wasm_bindgen};

    /// Returns a failure as the string a call throws.
    fn thrown(failure: Failure) -> JsValue {
        JsValue::from_str(&failure.0)
    }

    /// Returns a goal or a position the page gives as a number, or the
    /// failure thrown: see [`calls::index`].
    fn index(number: f64, what: &str) -> Result<u32, JsValue> {
        calls::index(number, what).map_err(thrown)
    }

    /// See [`calls::about`].
    #[wasm_bindgen]
    pub fn about() -> String {
        calls::about()
    }

    /// See [`calls::defaults`].
    #[wasm_bindgen]
    pub fn defaults() -> String {
        calls::defaults()
    }

    /// See [`calls::read`].
    #[wasm_bindgen]
    pub fn read(input: &str, settings: &str) -> Result<String, JsValue> {
        calls::read(input, settings).map_err(thrown)
    }

    /// See [`calls::read_as`].
    #[wasm_bindgen]
    pub fn read_as(input: &str, mode: &str, settings: &str) -> Result<String, JsValue> {
        calls::read_as(input, mode, settings).map_err(thrown)
    }

    /// See [`calls::start`].
    #[wasm_bindgen]
    pub fn start(sequent: &str, mode: &str, settings: &str) -> Result<String, JsValue> {
        calls::start(sequent, mode, settings).map_err(thrown)
    }

    /// See [`calls::view`].
    #[wasm_bindgen]
    pub fn view(session: &str, settings: &str) -> Result<String, JsValue> {
        calls::view(session, settings).map_err(thrown)
    }

    /// See [`calls::rules`].
    #[wasm_bindgen]
    pub fn rules(
        session: &str,
        goal: f64,
        position: f64,
        settings: &str,
    ) -> Result<String, JsValue> {
        let (goal, position) = (index(goal, "a goal")?, index(position, "a position")?);
        calls::rules(session, goal, position, settings).map_err(thrown)
    }

    /// See [`calls::split_passes`].
    #[wasm_bindgen]
    pub fn split_passes(
        session: &str,
        goal: f64,
        step: &str,
        settings: &str,
    ) -> Result<bool, JsValue> {
        calls::split_passes(session, index(goal, "a goal")?, step, settings).map_err(thrown)
    }

    /// See [`calls::apply`].
    #[wasm_bindgen]
    pub fn apply(session: &str, goal: f64, step: &str, settings: &str) -> Result<String, JsValue> {
        calls::apply(session, index(goal, "a goal")?, step, settings).map_err(thrown)
    }

    /// See [`calls::cut`].
    #[wasm_bindgen]
    pub fn cut(
        session: &str,
        goal: f64,
        formula: &str,
        left: &str,
        settings: &str,
    ) -> Result<String, JsValue> {
        calls::cut(session, index(goal, "a goal")?, formula, left, settings).map_err(thrown)
    }

    /// See [`calls::invert_all`].
    #[wasm_bindgen]
    pub fn invert_all(session: &str, settings: &str) -> Result<String, JsValue> {
        calls::invert_all(session, settings).map_err(thrown)
    }

    /// See [`calls::undo`].
    #[wasm_bindgen]
    pub fn undo(session: &str, settings: &str) -> Result<String, JsValue> {
        calls::undo(session, settings).map_err(thrown)
    }

    /// See [`calls::close`].
    #[wasm_bindgen]
    pub fn close(session: &str, goal: f64, settings: &str) -> Result<String, JsValue> {
        calls::close(session, index(goal, "a goal")?, settings).map_err(thrown)
    }

    /// See [`calls::fill`].
    #[wasm_bindgen]
    pub fn fill(session: &str, goal: f64, source: &str, settings: &str) -> Result<String, JsValue> {
        calls::fill(session, index(goal, "a goal")?, source, settings).map_err(thrown)
    }

    /// See [`calls::close_all`].
    #[wasm_bindgen]
    pub fn close_all(session: &str, settings: &str) -> Result<String, JsValue> {
        calls::close_all(session, settings).map_err(thrown)
    }

    /// See [`calls::hint`].
    #[wasm_bindgen]
    pub fn hint(session: &str, goal: f64, settings: &str) -> Result<String, JsValue> {
        calls::hint(session, index(goal, "a goal")?, settings).map_err(thrown)
    }

    /// See [`calls::proof`].
    #[wasm_bindgen]
    pub fn proof(session: &str, settings: &str) -> Result<String, JsValue> {
        calls::proof(session, settings).map_err(thrown)
    }

    /// See [`calls::prove`].
    #[wasm_bindgen]
    pub fn prove(sequent: &str, mode: &str, settings: &str) -> Result<String, JsValue> {
        calls::prove(sequent, mode, settings).map_err(thrown)
    }

    /// See [`calls::draw`].
    #[wasm_bindgen]
    pub fn draw(session: &str, settings: &str) -> Result<String, JsValue> {
        calls::draw(session, settings).map_err(thrown)
    }

    /// See [`calls::draw_net`].
    #[wasm_bindgen]
    pub fn draw_net(session: &str, settings: &str) -> Result<String, JsValue> {
        calls::draw_net(session, settings).map_err(thrown)
    }

    /// See [`calls::export`]. `export` is a reserved word of JavaScript,
    /// which wasm-bindgen would rename `_export`.
    #[wasm_bindgen(js_name = "export_as")]
    pub fn export(session: &str, format: &str, settings: &str) -> Result<String, JsValue> {
        calls::export(session, format, settings).map_err(thrown)
    }

    /// See [`calls::render`].
    #[wasm_bindgen]
    pub fn render(
        svg: &str,
        format: &str,
        font: &[u8],
        settings: &str,
    ) -> Result<Vec<u8>, JsValue> {
        calls::render(svg, format, font, settings).map_err(thrown)
    }

    /// See [`calls::certificate`].
    #[wasm_bindgen]
    pub fn certificate(document: &str, settings: &str) -> Result<String, JsValue> {
        calls::certificate(document, settings).map_err(thrown)
    }
}
