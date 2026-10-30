# Agent Guidelines for Geologia

This document sets architectural principles, development policies, and conventions for AI coding agents working on the `geologia` codebase.

---

## 1. Schema Modeling and Parsing Strictness

- **Strict schema parsing over loose fallbacks:**
  - `geologia` prioritizes strict schema validation so contract drift and malformed API payloads fail loudly with actionable deserialization errors.
  - **Do NOT add untagged catch-all variants** (e.g. `#[serde(untagged)] Unknown(Value)` or `Unknown(Map<String, Value>)`) to data enums such as `PartData`.
  - Untagged fallbacks silently match any valid JSON structure, which causes malformed payloads for *known* variants (e.g. a `functionCall` missing its required `name` field, or numeric values passed for text fields) to silently fall through to `Unknown` instead of raising a descriptive parse error.
  - Prefer closed enums and explicit structs. When new fields or variants are introduced upstream in the Gemini API, model them explicitly.

- **Upstream API alignment:**
  - Match types and field names against the official Gemini Generative Language REST `v1beta` discovery document (`https://generativelanguage.googleapis.com/$discovery/rest?version=v1beta`).
  - Use `Option<T>` with `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields to keep wire payloads concise and preserve backwards compatibility.

---

## 2. Command and Tool Usage Policy

- **Avoid shell commands for file inspection and mutation:**
  - **Never** use PowerShell or terminal commands (such as `Get-ChildItem`, `Get-Item`, `Remove-Item`, `dir`, `ls`, `rm`, `cat`, etc.) to discover, read, edit, or delete files.
  - **Always** use built-in tools (`read_file`, `write_file`, `edit_file`) for inspecting and modifying file contents.
  - Terminal commands are reserved for builds, tests, formatting, and lints (`cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`).

- **GitHub CLI:**
  - **Always** use the `gh` CLI for any GitHub URL, repository inspection, or fetching/managing PRs and issues.

---

## 3. Verification and Quality Standards

Before committing, pushing, or opening/updating PRs, ensure all of the following checks pass cleanly:
1. `cargo test` — all unit, integration, and doc-tests must pass.
2. `cargo clippy --all-targets -- -D warnings` — zero warnings across all crates, examples, and tests.
3. `cargo fmt --check` — all files must conform to standard Rust formatting.
