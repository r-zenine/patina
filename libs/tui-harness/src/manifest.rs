//! Machine-readable discovery types: the app manifest served by `--describe`.
//!
//! The manifest is the agent's entry point: instead of reading an app's
//! source (or trusting docs that can drift), an agent runs `--describe` and
//! gets the app's modes, keybindings, input notation, and the JSON Schema of
//! its state snapshot — asserted by the binary itself.

use schemars::JsonSchema;
use serde::Serialize;

use crate::input_parser;

/// Version of the manifest envelope emitted by `--describe`.
pub const MANIFEST_VERSION: u32 = 1;

/// Full manifest emitted by `--describe`.
///
/// Assembled by the harness (`build_manifest`): the app authors the
/// [`AppDescription`] part; the notation grammar and snapshot schema are
/// generated so they cannot drift.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AppManifest {
    /// Envelope version — bump on breaking manifest shape changes.
    pub manifest_version: u32,
    /// False when the app relies on the default `describe()` — lets agents
    /// distinguish "no discovery data" from an unsupported flag.
    pub described: bool,
    /// App name (empty when undescribed).
    pub app: String,
    /// App version (empty when undescribed).
    pub version: String,
    /// Input modes the app can be in.
    pub modes: Vec<ModeDoc>,
    /// Keybindings per mode.
    pub bindings: Vec<KeyBindingDoc>,
    /// Grammar of the compact input-sequence notation, generated from the
    /// parser's own key tables.
    pub notation: NotationDoc,
    /// JSON Schema of the app's state snapshot, generated via schemars.
    pub snapshot_schema: serde_json::Value,
}

/// App-authored discovery data returned by `ELMApp::describe`.
#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub struct AppDescription {
    /// App name.
    pub app: String,
    /// App version.
    pub version: String,
    /// Input modes the app can be in.
    pub modes: Vec<ModeDoc>,
    /// Keybindings per mode.
    pub bindings: Vec<KeyBindingDoc>,
}

/// One input mode of the app.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ModeDoc {
    /// Mode name as it appears in snapshots.
    pub name: String,
    /// What the mode is for.
    pub description: String,
}

/// One keybinding: keys (in input-sequence notation) → event, scoped to a mode.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct KeyBindingDoc {
    /// Mode this binding is active in.
    pub mode: String,
    /// Key aliases in input-sequence notation (e.g. `["j", "<Down>"]`).
    pub keys: Vec<String>,
    /// Event the binding produces.
    pub event: String,
    /// Human/agent-readable description.
    pub description: String,
}

/// A key that is meaningful in the app's *current* state.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Affordance {
    /// Key aliases in input-sequence notation.
    pub keys: Vec<String>,
    /// Event the key produces right now.
    pub event: String,
    /// What pressing it does.
    pub description: String,
}

/// Grammar of the compact input-sequence notation.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct NotationDoc {
    /// One-paragraph summary of the notation.
    pub description: String,
    /// Example sequences.
    pub examples: Vec<String>,
    /// Names accepted inside angle brackets (e.g. "Space", "F1").
    pub special_keys: Vec<String>,
    /// Modifier prefixes accepted inside angle brackets.
    pub modifiers: Vec<String>,
    /// Wait-step syntax.
    pub wait: String,
}

impl NotationDoc {
    /// Build the grammar doc from the parser's own tables — the parser and
    /// the documentation cannot disagree.
    pub fn grammar() -> Self {
        Self {
            description: "Compact vim-style key sequences: bare characters are \
                          individual key presses; special keys and modified keys \
                          use angle brackets (e.g. <Enter>, <C-j> for Ctrl+j, \
                          <C-S-x> for Ctrl+Shift+x)."
                .to_string(),
            examples: vec![
                "jjk<Space>".to_string(),
                "<Enter>nhello<Esc>".to_string(),
                "<Space><Wait:600>".to_string(),
            ],
            special_keys: input_parser::special_key_names(),
            modifiers: input_parser::modifier_names(),
            wait: "<Wait:N> sleeps N milliseconds and runs on_tick, so \
                   time-based behavior (e.g. timeouts) fires deterministically"
                .to_string(),
        }
    }
}

/// Assemble the full manifest for an app: app-authored description plus
/// generated notation grammar and snapshot schema.
pub fn build_manifest<M: crate::ELMApp>(app: &M) -> AppManifest {
    let description = app.describe();
    let described = description.is_some();
    let d = description.unwrap_or_default();
    let schema = schemars::schema_for!(M::Snapshot);

    AppManifest {
        manifest_version: MANIFEST_VERSION,
        described,
        app: d.app,
        version: d.version,
        modes: d.modes,
        bindings: d.bindings,
        notation: NotationDoc::grammar(),
        snapshot_schema: serde_json::to_value(schema)
            .expect("a schemars schema always serializes to JSON"),
    }
}
