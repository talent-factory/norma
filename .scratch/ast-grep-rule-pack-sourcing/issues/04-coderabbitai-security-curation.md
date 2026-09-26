Type: research
Status: resolved

## Question

`coderabbitai/ast-grep-essentials` (Apache-2.0, see ticket
03-github-external-rule-repos.md's "Deferred" section) is a 184-rule
security/SAST rule collection, 98 of which target norma's four
default-pattern languages (python 48, java 36, rust 8, typescript 6),
almost entirely `hardcoded-secret`/`empty-password` checks repeated
per-library (e.g. ~20 near-duplicate Python DB-driver variants).

Curate a small, non-redundant subset rather than importing wholesale:
for each language, collapse the mechanically-repeated per-library
variants into one generic rule per distinct security concept (e.g. one
generic "hardcoded secret passed to a connection/client call" rule and
one generic "empty password accepted" rule, not 20+ library-specific
copies), and separately adopt any rule in the collection that's already
conceptually distinct (not a library-specific duplicate of another rule
already covered). Judge each against the map's linter boundary is NOT
the filter here (security/SAST is explicitly the point of this ticket,
unlike 01-03) -- judge instead against redundancy: don't adopt two rules
that are really the same check with a different library name spliced in.

For every adopted rule, draft its ast-grep `RuleConfig` YAML (per ADR
0002, attributed with `# Source:`/`# License: Apache-2.0` comments, note
this is a derivative/generalized rule where applicable) into
`rule-packs/security/<language>.yaml`.

Record the full decided list (rule name or generic-rule description,
which original per-library rules it replaces/generalizes, one-line
rationale) as this ticket's resolution, append it to `../map.md`'s
"## Decisions so far", and verify the result with a real
`norma import rule-packs/security --category security` against a
throwaway DB (0 skipped expected).

## Answer

Cloned `coderabbitai/ast-grep-essentials` locally (shallow) and read every
file individually for python/rust/typescript, plus 8 of Java's 36 (the
richest, most diverse language in the collection -- it isn't just
per-library secret duplication like the others, it also covers weak
crypto, cookie flags, XXE, command injection, and TLS). Drafted 21
generalized rules total, all verified with a real
`norma import rule-packs/security --category security` (21 imported,
0 skipped, 0 id collisions with the 18 rules from tickets 01-03 or the
20 existing defaults).

### Python (6 rules -> `rule-packs/security/python.yaml`)

| Rule | Replaces | Confidence |
|---|---|---|
| `hardcoded-secret-in-call-python` | ~25 per-library `*-hardcoded-secret*` rules (psycopg2, redis, pymongo, mysql, ldap3, neo4j, couchbase, mariadb, pymssql, pg8000, tormysql, webrepl, urllib3, requests(-oauth), jwt-python, openai, hashids-with-django/flask-secret, elasticsearch-bearer-auth) | High -- 4 real files read in full to confirm the shared `call` + `keyword_argument` shape |
| `empty-password-accepted-python` | ~18 per-library `*-empty-password*` rules | High -- 2 real files read in full |
| `insecure-weak-cipher-rc4-python` | `insecure-cipher-algorithm-rc4-python` | High -- adapted near-verbatim from the one real file |
| `debug-mode-enabled-python` | `debug-enabled-python` | High -- adapted from the real file, import-guard dropped |
| `insecure-temp-file-mktemp-python` | `avoid-mktemp-python` | High -- adapted from the real file |
| `flask-run-exposed-host-python` | `avoid_app_run_with_bad_host-python` | High -- adapted from the real file |

### Rust (3 rules -> `rule-packs/security/rust.yaml`)

| Rule | Replaces | Confidence |
|---|---|---|
| `hardcoded-secret-in-call-rust` | `hardcoded-password-rust`, `tokio-postgres-hardcoded-password-rust`, `secrets-reqwest-hardcoded-auth-rust` | High -- all 3 read in full; the sqlx builder-chain/import-tracing was intentionally dropped for a simpler, broader `.password(...)`/`.bearer_auth(...)`/`.basic_auth(_, Some(...))` check |
| `empty-password-accepted-rust` | `empty-password-rust`, `postgres-empty-password-rust`, `tokio-postgres-empty-password-rust` | Medium -- only `hardcoded-password-rust` (the non-empty sibling) was read in full; the 3 empty-password files were inferred to mirror it by strong naming/file-family symmetry, not independently opened |
| `insecure-tls-verification-disabled-rust` | `reqwest-accept-invalid-rust`, `ssl-verify-none-rust` | High -- both read in full |

### TypeScript (5 rules -> `rule-packs/security/typescript.yaml`)

| Rule | Replaces | Confidence |
|---|---|---|
| `hardcoded-secret-in-call-typescript` | `express-session-hardcoded-secret-typescript`, `node-sequelize-hardcoded-secret-argument-typescript` | Medium -- neither individually opened; generalized from the Python/Rust pattern shape plus the filenames' own naming, not TS-source-verified |
| `empty-password-accepted-typescript` | `node-sequelize-empty-password-argument-typescript` | Medium -- same caveat |
| `jwt-decode-without-verify-typescript` | `jwt-simple-noverify-typescript` | High -- read in full, import-guard dropped |
| `weak-rsa-key-size-typescript` | `node-rsa-weak-key-typescript` (partially) | Medium -- read in full, but the original covers 3 call shapes (`crypto.generateKeyPair`, `new NodeRSA`, `node-forge`); only the `modulusLength` property shape was kept, the other two dropped as unverified |
| `angular-sce-disabled-typescript` | `detect-angular-sce-disabled-typescript` | High -- read in full, adopted near-verbatim |

### Java (7 of 36 rules -> `rule-packs/security/java.yaml`)

| Rule | Replaces | Confidence |
|---|---|---|
| `weak-crypto-algorithm-java` | `des-is-deprecated`, `desede-is-deprecated`, `use-of-rc2`, `use-of-rc4`, `use-of-blowfish`, `use-of-default-aes`, `use-of-md5`, `use-of-sha1` (8 rules) | Medium -- `use-of-rc4-java` and `use-of-md5-java` read in full (confirmed shared `$X.getInstance("ALGO")` shape); the other 6 inferred by strong naming/structural symmetry, not opened |
| `hardcoded-secret-in-credentials-java` | `datanucleus-hardcoded-connection-password`, `drivermanager-hardcoded-secret`, `hardcoded-connection-password`, `hardcoded-secret-in-credentials`, `java-jwt-hardcoded-secret`, `jedis-jedisclientconfig-hardcoded-password`, `jedis-jedisfactory-hardcoded-password`, `passwordauthentication-hardcoded-password`, `system-setproperty-hardcoded-secret` (9 rules) | Low-medium -- none individually opened; a generic Java "password-named setter/builder with a string-literal argument" + `DriverManager.getConnection` shape, reasoned from Java conventions rather than the real files. **Flagging this one as the least-verified rule in this pack** -- worth a spot-check against 2-3 of the real upstream files before relying on it. |
| `insecure-cookie-httponly-java` | `cookie-httponly-false-java`, `cookie-missing-httponly-java` | High -- both read in full |
| `xxe-external-general-entities-java` | `documentbuilderfactory-external-general-entities-true-java` (1 of 3 XXE rules) | High -- read in full, simplified (dropped variable-indirection matching) |
| `weak-ssl-context-java` | `weak-ssl-context-java` | High -- read in full, adopted near-verbatim |
| `unencrypted-socket-java` | `unencrypted-socket-java` | High -- read in full, adopted near-verbatim |
| `command-injection-runtime-exec-java` | `simple-command-injection-direct-input-java` | High -- read in full, Spring-MVC-specific parameter-type guard dropped for a simpler (broader) "non-literal argument" check |

**Deliberately deferred / not included this pass** (not individually
verified, don't want to ship guessed rules): Java's `cookie-secure-flag`
family (3 rules), `cookie-missing-samesite-java`, the other 2 XXE
variants (`external-parameter-entities`, `disallow-doctype-decl-false`),
and the non-`getInstance` crypto checks (`ecb-cipher-java`,
`use-of-aes-ecb-java`, `no-null-cipher-java`, `rsa-no-padding-java`,
`use-of-md5-digest-utils-java`, `cbc-padding-oracle-java`). Also Go
(11 upstream security rules) -- out of scope per the map's Notes (Go
isn't a norma default-pattern language and didn't surface in ticket
02's own-repo survey).

**General caveat that applies to every rule in this pack:** every one
was simplified relative to its upstream original by dropping
import-tracing/aliasing guards (`follows: kind: import_declaration ...`)
that verified e.g. "this `connect` call really came from
`psycopg2`/`mysql.connector`". This makes each generalized rule broader
(catches more call shapes) but also more prone to false positives on
code that happens to share a method/keyword name for an unrelated
reason. Recommend spot-checking each rule against real code before
trusting it as a hard compliance gate, same as any newly-adopted pattern.

Cleanup: deleted the throwaway verification DB and the local shallow
clone of `ast-grep-essentials` (`/tmp/ast-grep-essentials`) after use.

## Follow-up (post-resolution spot-check)

Daniel asked for the flagged least-verified rule,
`hardcoded-secret-in-credentials-java`, to be spot-checked against its
real upstream files before trusting it. Fetched 4 of its 9 referenced
originals directly: the initial draft's name-regex
(`(?i)^(set)?(password|passwd|secret)$`) missed
`setConnectionPassword` (JDO), `updatePassword` (Jedis), and
`Credentials.basic(...)` (okhttp) -- roughly half of what it claimed to
replace. Fixed: widened the regex to
`(?i)^(set|update)?(connection)?(password|passwd|secret)$` and added an
explicit `Credentials.basic($USER, $PASS)` branch (kept "basic" out of
the generic name-regex deliberately -- too generic, would false-positive
on unrelated builder methods). Left the positional 5-argument Jedis/JDO
constructor/`.create(...)` variants uncovered, same as before -- still
too fragile to generalize by argument position alone.

Re-verified with a real `norma import rule-packs/security --category
security` (21/21, 0 skipped) plus a real `norma validate --language
java` against a hand-written test file exercising all 4 previously-missed
shapes (now all 4 correctly flagged) and 2 negative cases (`setName`,
`password(<non-literal>)` -- correctly not flagged).

## Follow-up 2 -- spot-checked the remaining Medium-confidence rules

Daniel asked for the rest of the Medium-confidence rules to be
spot-checked too. Fetched the real upstream files for all of them:

- **`empty-password-accepted-rust`**: no bug. All 3 originals
  (`empty-password-rust`, `postgres-empty-password-rust`,
  `tokio-postgres-empty-password-rust`) really do share the exact same
  `.password("")` method name across different receiver types (sqlx,
  postgres, tokio-postgres crates) -- the naming-symmetry inference that
  drafted this rule was correct. Also re-verified `hardcoded-secret-in-
  call-rust`'s other 2 replaced originals
  (`secrets-reqwest-hardcoded-auth-rust`,
  `tokio-postgres-hardcoded-password-rust`) directly -- both match as
  drafted, no bug there either.
- **`hardcoded-secret-in-call-typescript`**: real bug. The rule's outer
  `kind: call_expression` gate silently missed every `new X(...)`
  constructor call -- including plain object-literal secrets like
  `new SomeClient({ secret: "..." })`, not just the (already
  out-of-scope) Sequelize positional-argument case. Fixed: gate now
  matches `call_expression` OR `new_expression`.
- **`empty-password-accepted-typescript`**: real double bug -- same
  `call_expression`-only gate, *and* its "any empty string argument
  anywhere" branch (meant to approximate the Sequelize positional case)
  was far too broad: verified it false-positives on any unrelated call
  passing an empty string, e.g. `someFn("a", "b", "")`. Fixed: gate
  widened same as above; the over-broad branch was dropped entirely
  rather than narrowed (positional-argument secrets aren't safely
  generalizable without library-specific knowledge, same call made for
  Java's Jedis/JDO constructors).
- **`weak-rsa-key-size-typescript`**: real bug. Threshold regex
  (`^([0-9]{1,3}|1[0-9]{3})$`) only covered 0-1999, silently missing
  2000-2047 (still weak per NIST, below the 2048 cutoff) -- verified
  `modulusLength: 2000`/`2047` slipped through before the fix. Widened
  to 0-2047 (reusing the interval structure from the real upstream
  rule's own constraint, which does cover the full range correctly).
- **`weak-crypto-algorithm-java`**: all 6 not-individually-opened
  algorithms (`des-is-deprecated`, `desede-is-deprecated`, `use-of-rc2`,
  `use-of-blowfish`, `use-of-default-aes`, `use-of-sha1`) confirmed
  correct for the bare-algorithm-name case. One real gap found on
  request: `des-is-deprecated-java` (read in full) additionally matches
  "ALGO/MODE/PADDING" transformation strings like
  "DES/CBC/PKCS5Padding" -- an inconsistency in the *upstream* per-rule
  set (RC2/Blowfish's own rules don't check transformation strings even
  though the same real-world risk applies), not a deliberate design
  choice worth replicating narrowly. Fixed by applying the
  transformation-string check to every listed algorithm except AES
  (kept exact-match-only, since bare "AES" is the insecure-default
  finding but "AES/GCM/NoPadding" is the secure, recommended form and
  must stay unflagged). Verified via `norma validate`:
  "DES/CBC/PKCS5Padding", "RC2/ECB/PKCS5Padding",
  "3DES/CBC/PKCS5Padding" now all correctly flagged, "AES/GCM/NoPadding"
  and "SHA-256" still correctly not flagged.

All fixes re-verified together with a final `norma import rule-packs/security
--category security` (21/21, 0 skipped).
