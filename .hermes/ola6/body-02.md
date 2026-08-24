# [Ola 6.02] feat-swal-52 — Standalone & Desktop Agent Protocol Connector in Rust

> Ola 6 — Standalone Cross-Platform Swal-Files & Autonomous Agentic Desktop Integration (Zero-Eww).
> Labels: `ola6`, `wave-6`

---

## Current State (MEDIBLE)
- `crates/swal-files/src/agent.rs` provides basic local prompt and file action generation.
- When running outside the SWAL Desktop environment (e.g. on Windows or generic Linux), there is no unified client that connects to Xavier Cognitive Memory (`:8006`), Hermes Agent (`:8100`), or OpenClaw MCP servers, or gracefully degrades to local offline heuristics.

## Desired State (DELTA)
- **Specific Addition**: Create `crates/swal-files/src/agent_protocol.rs`:
  - Structs & Enums:
    - `AgentEndpointConfig`: `http_endpoint: String`, `mcp_port: u16`, `auth_token: Option<String>`, `fallback_to_local: bool`.
    - `AgentQueryRequest`: `query: String`, `current_directory: String`, `selected_files: Vec<String>`, `intent: String`.
    - `AgentQueryResponse`: `summary: String`, `suggested_actions: Vec<AgentActionItem>`, `referenced_paths: Vec<String>`, `confidence: f32`, `source_agent: String`.
    - `AgentActionItem`: `id: String`, `title: String`, `action_type: String`, `params: std::collections::HashMap<String, String>`.
    - `AgentProtocolClient`: Client struct with methods:
      - `new(config: AgentEndpointConfig) -> Self`
      - `discover_local_nodes() -> Vec<String>` (Discovers running Xavier/Hermes instances)
      - `is_connected(&self) -> bool`
      - `send_query(&self, req: &AgentQueryRequest) -> Result<AgentQueryResponse, String>` (Performs async/sync query or falls back to local heuristic response when offline)
      - `execute_action(&self, action: &AgentActionItem) -> Result<String, String>`
  - **Embedded Unit Tests**: Include comprehensive unit tests testing endpoint initialization, request serialization, offline fallback mode, mock agent responses, and action dispatch with 100% test coverage.
- **File Target**: `crates/swal-files/src/agent_protocol.rs`

## Web Research Required
1. search: "rust async http client reqwest serde json rpc"
2. search: "rust graceful offline fallback architecture"
3. search: "model context protocol mcp client rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `cargo test -p swal-files` — all unit tests pass
- [ ] `grep -rn "AgentProtocolClient" crates/swal-files/src/agent_protocol.rs` >= 1 match
- [ ] `grep -rn "AgentQueryRequest" crates/swal-files/src/agent_protocol.rs` >= 1 match

## Files to Modify
| File | Current State | Change | Risk |
|------|--------------|--------|------|
| `crates/swal-files/src/agent_protocol.rs` | Non-existent | [NEW] Agent protocol connector with offline fallback and 100% unit tests | LOW |

## DO NOT touch
- `crates/swal-files/src/lib.rs` — core crate
- `.gitcore/features.json` — reconciled at wave end

## Anti-Hallucination Guard
1. READ `crates/swal-files/src/agent.rs` first.
2. Implement pure, safe Rust 2021 code without unhandled panics and with complete unit tests.

## Merge Order
- **Merge order within wave:** 2
- **Expected effort:** Small (<25m)
- **Parallel with:** All other wave issues (disjoint file islands)
