---
Status: accepted
---

# Pattern rows are single-language; `rule` stores the full ast-grep RuleConfig YAML

The original scaffold modeled `Pattern.languages: Vec<String>` — one pattern, several languages, one shared `rule` string. A verified probe against the real `ast-grep-config` crate (see [`04-ast-grep-probe.rs`](../../.scratch/norma-architecture/issues/04-ast-grep-probe.rs)) showed this doesn't match how ast-grep actually works: `RuleConfig<L>` takes a single `language: L`, not a list — pattern syntax is inherently language-specific (Java and TypeScript parse differently), so one `rule` string cannot validly apply to several languages at once.

Decided instead: **one `Pattern` row = one language**. A concept that should hold across several languages (e.g. "no debug prints") becomes several rows — one per language, each with its own `rule` text — related only loosely by a shared `name`, not by a data-level relationship. This removes the need for a languages-join-table or JSON-array column entirely; `language` becomes a single indexed `TEXT` column, and the substring-match bug in the original scaffold (`LIKE '%java%'` matching `"javascript"`) disappears as a side effect.

Further: the `rule` column stores the **complete** ast-grep `RuleConfig` YAML document (`id`, `message`, `severity`, `language`, `rule:`, optionally `fix:`) rather than just the inner match clause. `severity` and `language` are extracted from this YAML at register time (via `ast_grep_config::from_yaml_string`) and mirrored into their own indexed columns — they're never accepted as separate, independently-settable inputs, so the YAML and the columns can't drift apart. This also means a rule can be copy-pasted directly from ast-grep's own documentation or CLI output. norma's own `description` field stays separate from the YAML's `message` (which ast-grep documents as "should be single line and concise") to allow a longer, pedagogical explanation.

The scaffold's separate `rewrite: Option<String>` field is dropped — ast-grep's `RuleConfig` already supports a `fix:` key inside the same YAML, so it belongs there instead of a parallel column.
