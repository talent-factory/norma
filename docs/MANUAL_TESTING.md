# Manual Testing Guide

A step-by-step checklist for exercising norma by hand -- the CLI, the MCP
server, and the pre-commit hook -- on top of the automated test suite
(`cargo test`). Use this after pulling changes, before a release, or
whenever you want to see norma actually catch something rather than just
trust the unit tests.

Everything below uses a scratch database (`--db /tmp/norma-manual-test.db`)
so it never touches your real `$HOME/.local/share/norma/norma.db` registry
or leaves state behind. Delete it any time with:

```bash
rm -f /tmp/norma-manual-test.db
```

## 0. Prerequisites

```bash
cd norma
cargo --version   # needs the toolchain pinned in rust-toolchain.toml (stable)
```

## 1. Automated tests still pass

Run this first -- if anything here is red, the manual steps below aren't
worth doing yet.

```bash
cargo test
```

Expected: every test in `src/` (unit tests per module) and `tests/`
(`dogfooding.rs`, `gof_patterns.rs`) passes, `0 failed`.

(All of this document's steps also have a `just` shortcut for the
everyday case -- run `just` to list them. They use your real
`$HOME/.local/share/norma/norma.db`, not a scratch database, so they're
for quick day-to-day checks rather than the isolated, repeatable
verification this document is for.)

## 2. Build the binary

```bash
cargo build --release
```

Expected: `Finished` with no errors. The binary is at
`target/release/norma`; the commands below use `cargo run --` instead so
you don't have to remember that path, but swap in the release binary if
you want to measure real startup time.

## 3. CLI: `list-patterns`

```bash
cargo run -- --db /tmp/norma-manual-test.db list-patterns
```

Expected: one line per registered pattern (20 total: 4 MVP "no debug
print" patterns plus 16 GoF patterns), each shaped like:

```
no-debug-print-java          java       [warning] No Debug Print
```

The database file is created and seeded on first run -- run the command
again and the list must be identical (seeding is idempotent, it only
happens once per database).

## 4. CLI: `validate` finds a real violation

```bash
mkdir -p /tmp/norma-manual-test
printf 'fn main() {\n    println!("debug");\n}\n' > /tmp/norma-manual-test/dirty.rs

cargo run -- --db /tmp/norma-manual-test.db validate \
  --language rust /tmp/norma-manual-test/dirty.rs
echo "exit code: $?"
```

Expected: one violation line pointing at the `println!` call, a summary
line (`1 violation(s) (0 informational), score 0.80, 5 pattern(s) checked
(... ms)`), and **exit code 1**.

## 5. CLI: `validate` reports a clean pass

```bash
printf 'fn main() {\n    tracing::info!("fine");\n}\n' > /tmp/norma-manual-test/clean.rs

cargo run -- --db /tmp/norma-manual-test.db validate \
  --language rust /tmp/norma-manual-test/clean.rs
echo "exit code: $?"
```

Expected: `... no violations (... ms)` and **exit code 0**.

## 6. CLI: multiple files in one call

```bash
cargo run -- --db /tmp/norma-manual-test.db validate \
  --language rust /tmp/norma-manual-test/dirty.rs /tmp/norma-manual-test/clean.rs
echo "exit code: $?"
```

Expected: both files reported, `dirty.rs` shows the violation, `clean.rs`
shows none, and the overall exit code is **1** (any file with violations
fails the whole run -- this is what makes the pre-commit hook batch-safe).

## 7. CLI: `--json` output

```bash
cargo run -- --db /tmp/norma-manual-test.db validate \
  --language rust --json /tmp/norma-manual-test/dirty.rs /tmp/norma-manual-test/clean.rs \
  | python3 -m json.tool
```

Expected: a JSON array with exactly two objects (`{"file": ..., "result":
{"violations": [...], "passed": ..., "score": ..., "checked_patterns":
..., "duration_ms": ...}}`), one per file, in the order given on the
command line.

## 8. CLI: an unsupported language fails loudly

Since TF-893, all 28 languages `ast-grep-language` ships (including `go`,
tested as the "unsupported" example here before) are valid -- so this now
needs a genuinely unrecognized/misspelled one to still fail:

```bash
cargo run -- --db /tmp/norma-manual-test.db validate \
  --language cobol /tmp/norma-manual-test/clean.rs
echo "exit code: $?"
```

Expected: an error mentioning `unsupported language` and norma's full
28-language support list, **not** a silent "0 violations" pass. Exit code
non-zero. (`go` itself now succeeds -- see step 10's `get_pattern_checklist`
sub-step for what it reports: registrable, no default patterns yet.)

## 8a. CLI: `--fix` applies fixes and reports conflicts

```bash
printf 'fn main() {\n    println!("debug");\n}\n' > /tmp/norma-manual-test/fixable.rs

cargo run -- --db /tmp/norma-manual-test.db validate \
  --language rust --fix /tmp/norma-manual-test/fixable.rs
echo "exit code: $?"
cat /tmp/norma-manual-test/fixable.rs
```

The shipped `no-debug-print-rust` pattern has no `fix:` of its own, so
nothing is rewritten here and the file is unchanged -- this step only
confirms `--fix` is a no-op (not an error) against a pattern with no fix
to apply. To see an actual rewrite, register a pattern with a `fix:`
first and re-run against code it matches:

```bash
cat > /tmp/norma-manual-test/register-fix.json <<'JSON'
{"name":"No await in Promise.all","description":"test fixture","category":"code-quality","rule":"id: no-await-in-promise-all\nseverity: error\nlanguage: TypeScript\nmessage: No await in Promise.all\nrule:\n  pattern: await $A\n  inside:\n    pattern: Promise.all($_)\n    stopBy:\n      not: { any: [{ kind: array }, { kind: arguments }] }\nfix: $A\n"}
JSON
printf 'Promise.all([await doA(), doB()]);\n' > /tmp/norma-manual-test/fixable.ts
```

then, via the MCP Inspector (step 10) or `curl`, call `register_pattern`
with that JSON's fields and re-run:

```bash
cargo run -- --db /tmp/norma-manual-test.db validate \
  --language typescript --fix /tmp/norma-manual-test/fixable.ts
cat /tmp/norma-manual-test/fixable.ts
```

Expected: `applied 1 fix(es)` on stdout, exit code depends on whether the
rewritten file still has violations (it shouldn't, here), and the file's
`await doA()` is rewritten to `doA()`. Re-running `validate` (no `--fix`)
against the same file now reports **no violations** -- the fix actually
landed. If two enabled patterns' fixes ever overlap on the same code, you
should instead see a `[warning] fix conflict -- overlapping fixes from
..., ... were not applied` line and neither fix applied.

## 8b. CLI: `import` bulk-imports a rule directory

```bash
mkdir -p /tmp/norma-manual-test/rules
cat > /tmp/norma-manual-test/rules/no-await.yml <<'YAML'
id: no-await-in-promise-all
severity: error
language: TypeScript
message: No await in Promise.all
rule:
  pattern: await $A
  inside:
    pattern: Promise.all($_)
    stopBy:
      not: { any: [{ kind: array }, { kind: arguments }] }
fix: $A
YAML
cat > /tmp/norma-manual-test/rules/broken.yml <<'YAML'
language: Cobol
rule:
  pattern: DISPLAY $X
YAML

cargo run -- --db /tmp/norma-manual-test.db import \
  --category adopted-from-catalog /tmp/norma-manual-test/rules
echo "exit code: $?"
```

Expected: `imported no-await-in-promise-all [typescript]`, a `[warning]
skipped ...: unsupported language ...` line for `broken.yml`, a summary
(`1 imported, 1 skipped`), and a **non-zero exit code** (any skip fails
the run, same convention as `validate`'s "any violation fails the run").
Confirm it actually landed: `cargo run -- --db /tmp/norma-manual-test.db
list-patterns | grep no-await-in-promise-all`.

## 9. CLI: `--db` / `$NORMA_DB` override

```bash
# --db wins even if $NORMA_DB is also set
NORMA_DB=/tmp/should-not-be-used.db cargo run -- --db /tmp/norma-manual-test.db list-patterns

# $NORMA_DB is honored when --db is absent
NORMA_DB=/tmp/norma-manual-test.db cargo run -- list-patterns
```

Expected: both commands list the same patterns as step 3;
`/tmp/should-not-be-used.db` is never created.

## 10. MCP server: smoke test with the MCP Inspector

The easiest way to drive the server interactively (needs Node.js):

```bash
npx @modelcontextprotocol/inspector -- cargo run -- --db /tmp/norma-manual-test.db serve
```

Note the leading `-- ` right after `inspector`: the Inspector's own launcher
splits its argv on the *first* `--` it finds (to separate its own flags from
the ad-hoc server command) and then drops that token instead of forwarding
it. With only one `--` (i.e. `inspector cargo run -- --db ... serve`), that's
the one meant for `cargo run`, so it gets eaten and cargo sees a bare
`--db` it doesn't recognize (`error: unexpected argument '--db' found`,
visible as a "Failed" server in the UI). The extra `--` sacrifices itself to
the Inspector's split, letting the real one reach `cargo` intact.

This opens a local web UI. From there:

1. Connect (it should show `norma` as the server name).
2. Call **`list_patterns`** with no arguments -- expect the same patterns
   as step 3, as a JSON array.
3. Call **`validate_pattern_compliance`** with
   `{"code": "fn main() { println!(\"debug\"); }", "language": "rust"}`
   -- expect one violation.
4. Call **`get_pattern_checklist`** with `{"language": "python"}` --
   expect `{"patterns": [...]}` with only Python-language patterns and no
   `coverage_warning` field. Then call it again with `{"language": "go"}`
   (registrable since TF-893, but shipped with no default patterns) --
   expect `{"patterns": [], "coverage_warning": "no enabled patterns are
   registered for language \"go\""}`, not a bare empty array.
5. Call **`register_pattern`** with a deliberately broken `rule` (e.g.
   `{"name": "Broken", "description": "d", "rule": "not: valid: yaml"}`)
   -- expect an error response, not a silently accepted pattern (check
   with `list_patterns` afterwards that nothing new was added).
6. Call **`apply_pattern_fix`** with
   `{"code": "fn main() { println!(\"debug\"); }", "language": "rust"}`
   -- expect the rewritten source back (unchanged here, since the shipped
   `no-debug-print-rust` pattern has no `fix:`) and confirm the tool never
   touches disk (nothing under `/tmp/norma-manual-test.db`'s directory
   changes).
7. Call **`test_pattern`** with
   `{"rule": "id: no-debug-print-java\nmessage: test\nseverity: warning\nlanguage: Java\nrule:\n  pattern: System.out.println($$$ARGS)\n", "code": "System.out.println(\"hi\");"}`
   -- expect one match back (`passed: false`, meaning "the rule matched",
   not "something went wrong" -- see the tool's own description). Then
   call **`list_patterns`** and confirm nothing new was registered --
   `test_pattern` never touches the store.
8. Call **`import_rules`** with
   `{"yaml": "id: no-debug-print-elixir\nseverity: warning\nlanguage: Elixir\nmessage: test\nrule:\n  pattern: IO.puts($$$ARGS)\n"}`
   -- expect `{"imported": [...one pattern...], "skipped": []}`. Then call
   it again with the same `yaml` plus a second, malformed document
   appended (`"...\n---\nlanguage: Cobol\nrule:\n  pattern: DISPLAY $X\n"`)
   -- expect the first document to report as imported again (upsert, not
   duplicated) and the second as `skipped` with an `unsupported language`
   reason.

## 11. MCP server: smoke test without Node.js (raw stdio)

norma's MCP server speaks newline-delimited JSON-RPC 2.0 over stdio: send
one JSON object per line on stdin, read one JSON object per line back on
stdout. This sends an `initialize` handshake followed by a
`tools/call` for `list_patterns`, then closes stdin so the server exits:

```bash
{
  printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"manual-test","version":"0.0.0"}}}\n'
  printf '{"jsonrpc":"2.0","method":"notifications/initialized"}\n'
  printf '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_patterns","arguments":{}}}\n'
} | cargo run -- --db /tmp/norma-manual-test.db serve
```

Expected: two JSON-RPC responses on stdout (`id: 1` with the server's
capabilities, `id: 2` with the pattern list as `content`), and **no log
lines mixed into stdout** -- norma sends all its `tracing` output to
stderr precisely so this doesn't corrupt the protocol stream (see the
comment in `src/main.rs`). If you see anything other than JSON-RPC on
stdout, that's a real bug, not a formatting quirk of this test.

## 12. Pre-commit hook

```bash
cargo install --path . --force   # so `norma` is on PATH, as the hook requires
git stash -u   # keep this a clean check against your actual working tree, if needed
cd /tmp && rm -rf norma-hook-test && mkdir norma-hook-test && cd norma-hook-test
git init -q
cp /path/to/norma/.pre-commit-config.yaml .
printf 'fn main() {\n    println!("debug");\n}\n' > bad.rs
git add bad.rs .pre-commit-config.yaml
pre-commit install
git commit -m "test"
echo "exit code: $?"
```

Expected: the commit is **rejected**, with `norma`'s violation output for
`bad.rs` shown in the pre-commit failure log. Fix `bad.rs` (e.g. replace
`println!` with `tracing::info!`), `git add` it again, and the same
commit command should now succeed.

(Replace `/path/to/norma` with this repo's actual path. Clean up
afterwards with `cd .. && rm -rf norma-hook-test`.)

## 13. Dogfooding: norma validates its own source

```bash
cargo test --test dogfooding -- --nocapture
```

Expected: `norma_has_no_debug_prints_in_its_own_source ... ok`. If this
ever fails, it means a real `println!` crept into `src/` (excluding
`main.rs`, whose `println!` calls are the CLI's intended stdout output) --
fix the source, don't weaken the test.

## 14. Per-id seeding: an upgraded database picks up new patterns without losing customizations

`PatternStore::seed_defaults` inserts every default pattern whose id
isn't already a row -- it never touches a row that already exists. This
simulates an old (or user-edited) database and confirms both halves of
that guarantee:

```bash
rm -f /tmp/norma-upgrade-test.db

# Seed a fresh database, then strip it down to one row and rename it --
# standing in for an old install or a user's own edit.
cargo run -- --db /tmp/norma-upgrade-test.db list-patterns >/dev/null
sqlite3 /tmp/norma-upgrade-test.db "DELETE FROM patterns WHERE id != 'no-debug-print-rust';"
sqlite3 /tmp/norma-upgrade-test.db "UPDATE patterns SET name = 'My Custom Name' WHERE id = 'no-debug-print-rust';"

# Re-run norma -- seed_defaults must fill in the other 19 patterns...
cargo run -- --db /tmp/norma-upgrade-test.db list-patterns | wc -l

# ...while leaving the customized row completely untouched.
sqlite3 /tmp/norma-upgrade-test.db "SELECT name FROM patterns WHERE id = 'no-debug-print-rust';"
```

Expected: `20` from the `list-patterns | wc -l` line, and `My Custom
Name` from the final `sqlite3` query -- not the shipped default's real
name ("No Debug Print").

```bash
rm -f /tmp/norma-upgrade-test.db
```

## Cleanup

```bash
rm -f /tmp/norma-manual-test.db /tmp/should-not-be-used.db
rm -rf /tmp/norma-manual-test /tmp/norma-hook-test
```
