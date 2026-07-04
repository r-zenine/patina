# Context Handoff - Phase 4 Design (REPL Protocol)

## What Problem Are We Solving
One-shot CLI exploration replays the whole key prefix every invocation —
O(n²) and the reason tests encode long magic strings. The roadmap deferred
the exact protocol shape (commands, error semantics, envelope) to this gate.

## Design Overview
NDJSON request/response over stdio. Four commands: `keys` (apply an input
sequence), `render` (re-observe), `describe` (manifest), `quit`. Every
observation response is one compact line carrying `v:1, ok, state, visual,
affordances` — act and observe legal moves in a single round trip. Errors
(malformed JSON, unknown cmd, bad sequence, app error mid-sequence) never
end the session; `keys` errors report `applied:N` since partial application
mutates state. EOF exits cleanly; `should_quit` is observable, not fatal.
Rejected: request ids/pipelining (stdio orders), pretty JSON (breaks
framing), auto-exit on quit-flag (destroys observable state), MCP (D005).

## Reading Guide
Implementers: "How It Works" is the normative protocol (request/response
shapes and error table); "Implementation Guidance" names the exact reuse
points (`run_repl_io` generic over streams, `InputStep::apply`,
`build_manifest`). Success criterion: a scripted 3-command session against
review-tui with persisting cursor state.
