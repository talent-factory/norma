# Changelog

All notable changes to this project. Generated automatically by
[git-cliff](https://github.com/orhun/git-cliff) from the commit history on
every merge to `main` -- **do not edit by hand**, changes will be
overwritten on the next merge.

## [Unreleased]

### 🚀 Features

- Bulk-import existing ast-grep rule sets ([TF-894](https://linear.app/talent-factory/issue/TF-894)) ([a7dd66d](https://github.com/talent-factory/norma/commit/a7dd66d2a0e2ec82f157f888846d13a3ef7f2de6))

- Generically unlock all 28 SupportLang languages ([TF-893](https://linear.app/talent-factory/issue/TF-893)) ([0a49601](https://github.com/talent-factory/norma/commit/0a49601ef0bf380875d00aa908c9db0fed702b16))

- Add test_pattern MCP tool for dry-running rules ([TF-891](https://linear.app/talent-factory/issue/TF-891)) ([b5a7c03](https://github.com/talent-factory/norma/commit/b5a7c03487103416fdd82d6fc30157eb23e48f3f))

- Use RuleConfig's fix/fixer for suggested and applied fixes ([TF-890](https://linear.app/talent-factory/issue/TF-890)) ([4cb21b1](https://github.com/talent-factory/norma/commit/4cb21b1fe30fb1fa98ad3f18e3f89ce5549ebd0e))


### 🐛 Bug Fixes

- Address PR #8 multi-agent review findings ([TF-894](https://linear.app/talent-factory/issue/TF-894)) ([1d4ebf6](https://github.com/talent-factory/norma/commit/1d4ebf65621ed10c7b11019f5cd7d868465c33d9))

- Get_pattern_checklist no longer confuses "no coverage" with "compliant" ([aba1de3](https://github.com/talent-factory/norma/commit/aba1de32530bad17e35465ad42149f7f3281c0dc))

- Address PR #7 review findings ([TF-893](https://linear.app/talent-factory/issue/TF-893)) ([3c0cc17](https://github.com/talent-factory/norma/commit/3c0cc17520247edb9c150487dd0f53fdb843452a))

- Address review findings on test_pattern ([TF-891](https://linear.app/talent-factory/issue/TF-891)) ([fd2ac22](https://github.com/talent-factory/norma/commit/fd2ac220503c420d99b9843a8c5eecc88c6f4dc3))

- Don't self-conflict on nested matches, surface skipped rules (TF-890 review) ([67d1c6b](https://github.com/talent-factory/norma/commit/67d1c6b83a15a5899f3ba57d32dcf6b2c45e1adb))


### 📚 Documentation

- 📚 docs: CLAUDE.md für Claude Code Sessions anlegen ([febdc0d](https://github.com/talent-factory/norma/commit/febdc0d975d4ab5b09fb4c577e671c6f8a11af80))

- Bring DEVELOPMENT.md and MANUAL_TESTING.md in line with TF-890..894 ([c7b5435](https://github.com/talent-factory/norma/commit/c7b5435a72b84f217b995c8c17bf88dad1b9c405))

- Fix maintainer email typo in the norma-mvp plan doc ([75c7639](https://github.com/talent-factory/norma/commit/75c76398a2d42ee88573f89eb15bedaad62816e1))

- Fix maintainer email typo (daniel@talentfactory.ch -> daniel.senften@talent-factory.ch) ([72d86c6](https://github.com/talent-factory/norma/commit/72d86c6fa990c033e3d62df846ad3479a801b65a))

- Position norma vs. ast-grep-mcp, add rule-adoption example ([bd09461](https://github.com/talent-factory/norma/commit/bd09461a688ceaab97fd634c70992ce2411429d0))


### 🧭 Design Decisions

- Resolve ad-hoc-ephemeral-search ticket, map complete ([e744d0d](https://github.com/talent-factory/norma/commit/e744d0d8f3fe9d312fbfb6e2256135d118479b2e))

- Resolve bulk-import-existing-rules ticket ([b6e0b53](https://github.com/talent-factory/norma/commit/b6e0b531b47c825ccdeb74e5eca812e9732ebcd1))

- Resolve language-coverage-expansion ticket ([f4f6de2](https://github.com/talent-factory/norma/commit/f4f6de22fa529c44958a473df58a9960ac356d47))

- Resolve rule-testing/AST-debug-tooling ticket ([0460337](https://github.com/talent-factory/norma/commit/046033722f160e9fdef1e02ca8e2ecae667d3e41))

- Chart ast-grep feature-parity map, resolve autofix ticket ([580b2b0](https://github.com/talent-factory/norma/commit/580b2b0b52015c1d6c299997f823cd8aaec152ec))


### 🔩 Other Changes

- Pin register_pattern's category schema with a regression test ([8a6f863](https://github.com/talent-factory/norma/commit/8a6f863942a5a7d9ea1213927c56aae39ed57929))

- Render register_pattern's category as anyOf, not type array ([b4337e7](https://github.com/talent-factory/norma/commit/b4337e70c7c79f071d4a01b2b51e28c7dfc09e8c))

- Respect $XDG_DATA_HOME before falling back to $HOME/.local/share ([a1b5d54](https://github.com/talent-factory/norma/commit/a1b5d54885d9ebc61cd28f80245b59d100089eda))

- Sync manual testing guide with GoF v2 and per-id seeding ([8a3925f](https://github.com/talent-factory/norma/commit/8a3925f335d1ac4025f6b4a9383d2801fd9b9c58))

- Add a grouped justfile task runner ([532b408](https://github.com/talent-factory/norma/commit/532b408cdb075a3d7d4c087ae32b2f3e3b70fec2))

- Per-id pattern seeding so upgrades stop silently missing new defaults ([ee39b8f](https://github.com/talent-factory/norma/commit/ee39b8f34a73627019f17ad28cdf4d04f99318a4))

- Flag superseded Singleton YAML in the plan, add static-mut regression test ([87e685c](https://github.com/talent-factory/norma/commit/87e685c4ac00f21e1b1a1e5491405979539d96ba))

- Widen singleton-quality-rust back to real Singleton shapes ([fab8256](https://github.com/talent-factory/norma/commit/fab8256d751ae761249110e7b8f755ed5f5edbbe))

- Correct design-spec inaccuracies and polish doc comments ([d8b8aec](https://github.com/talent-factory/norma/commit/d8b8aec0d6f16346d8f9fdbc581c8e64a727b395))

- Split human-readable summary into blocking vs. informational counts ([39c1ea7](https://github.com/talent-factory/norma/commit/39c1ea760d5d33b6614f95dbef119e04baf82eb9))

- Fix singleton-quality rule anchoring and constructor-detection gaps ([5cb4c07](https://github.com/talent-factory/norma/commit/5cb4c070770dde999d1055c276d07359209f67c6))

- Cargo fmt pattern_engine.rs ([b487a1f](https://github.com/talent-factory/norma/commit/b487a1f1fe3b1498698adae54b71f798357538f8))

- Add missing gof_patterns.rs entry to README's project tree ([895325b](https://github.com/talent-factory/norma/commit/895325ba7d647fa685970cebc8da6a3fce01e086))

- Update QUICKSTART's stale four-MVP-patterns section ([7def865](https://github.com/talent-factory/norma/commit/7def86525a679337055fe172f0197acd3be5b470))

- Document that seed_defaults never reaches existing databases ([4bf69a5](https://github.com/talent-factory/norma/commit/4bf69a55a9165e720c5597b82a20938fa0748b60))

- Document Python Factory/Strategy rule collision + add regression test ([2171b8d](https://github.com/talent-factory/norma/commit/2171b8d685459bbc5f49347e51de141a74e8ce69))

- Document the GoF pattern set v2 in DEVELOPMENT.md and README.md ([b477658](https://github.com/talent-factory/norma/commit/b4776586340ef4d840ccb7f64fd05a65064c4cc6))

- Update integration tests to reflect 20 default patterns (not 4) ([70d1183](https://github.com/talent-factory/norma/commit/70d11836aa3ebd35cbc65313f97775f429670fbe))

- Assert the final GoF v2 pattern set shape (20 patterns, category-per-id-prefix) ([bc11f28](https://github.com/talent-factory/norma/commit/bc11f288e4ffdc549b11d9b2b367f4460a43b5ea))

- Add Strategy Overuse (Type Switch) default patterns (GoF v2) ([aaaaf67](https://github.com/talent-factory/norma/commit/aaaaf67991cdc49520627c1ad9cac62598aeaf2f))

- Add Observer Presence default patterns (GoF v2, info severity) ([0dac773](https://github.com/talent-factory/norma/commit/0dac773aade8409eca3ef6d1563a02b118c618c9))

- Add Factory Overuse (Type Switch) default patterns (GoF v2) ([43058f9](https://github.com/talent-factory/norma/commit/43058f9f976a150e3a15c17f9cdfc1ee539332d1))

- Add Singleton Implementation Quality default patterns (GoF v2) ([2be574d](https://github.com/talent-factory/norma/commit/2be574d888ad3dc111de7bdf0dffcb3cefea4f2f))

- Generalize default_patterns coverage test ahead of the GoF v2 additions ([f23638d](https://github.com/talent-factory/norma/commit/f23638dc414a47c05ec9129b3edb7e0f13ddbf4d))

- Make validate's score/passed severity-aware (info matches don't fail a run) ([a7db797](https://github.com/talent-factory/norma/commit/a7db7976775c6257736a6302f11740c075fdcd02))

- Add manual testing guide (CLI, MCP server, pre-commit hook) ([2840c4c](https://github.com/talent-factory/norma/commit/2840c4ca05fdf7409b0cdf0feb817567c1ada2c2))

- Add GoF pattern set v2 implementation plan ([4d638ff](https://github.com/talent-factory/norma/commit/4d638ff454f65dd98aed290f8762eef0f979175d))

- Add GoF pattern set v2 design (Ticket 06 + spec) ([deacb91](https://github.com/talent-factory/norma/commit/deacb917f1114eeed85ceb3547b515d2ea20e89c))

- Fix stale docs: remove references to the removed Pattern::new/rewrite/ ([2d22fcf](https://github.com/talent-factory/norma/commit/2d22fcf60dee5adf599a2af23dd1af7ba0ada48a))

- Extract validate_files/resolve_db_path into cli.rs so main.rs's CLI ([32aaf61](https://github.com/talent-factory/norma/commit/32aaf61e51df346cc6fd76aa56f25777c1ead345))

- Mcp_server: log tool failures, distinguish invalid input from storage ([1e67e45](https://github.com/talent-factory/norma/commit/1e67e45fdeaa32ee5a533acc34ca52b11e0f48eb))

- Enforce ADR 0002 structurally; make ValidationResult distinguish ([3041797](https://github.com/talent-factory/norma/commit/30417972de38fce5d374815841e1c32d58a6b207))

- Exclude non-source files from the pre-commit hook ([863882f](https://github.com/talent-factory/norma/commit/863882fcf78484781f08b8b0573d6c1360b2a5d6))

- Rewrite the stale QUICKSTART and align README with the CLI ([b4728d6](https://github.com/talent-factory/norma/commit/b4728d61615f8a5b3547bdb0db2ba27d5fe9189f))

- Fix stdio log corruption, multi-file CLI, language checks, DB path ([4d84bbf](https://github.com/talent-factory/norma/commit/4d84bbff4906f2022c7905a7cf2d219fedb04d07))

- Add pre-commit hook template, update README/DEVELOPMENT docs ([7d452ab](https://github.com/talent-factory/norma/commit/7d452ab0c71f490cce23d35071879ed4f0336e19))

- Revert main.rs println! rewrite, skip main.rs in dogfooding scan ([74b61be](https://github.com/talent-factory/norma/commit/74b61be7665387a5f7712f7cc55542a9cf1e7020))

- Add dogfooding test and remove debug prints ([34c0dea](https://github.com/talent-factory/norma/commit/34c0dea9e7d0d92139dad9dc6da78ce7ba871683))

- Wire CLI + MCP server together in main.rs ([2fbb1de](https://github.com/talent-factory/norma/commit/2fbb1dea74055f8b55adcd46b793dd6da8ec18df))

- Add clap CLI surface (serve/validate/list-patterns) ([c4faf0d](https://github.com/talent-factory/norma/commit/c4faf0d3b8017d70273bddfc663e09fe5c4bff2c))

- Add rmcp tool server with the 4 MCP tools ([d5f63d5](https://github.com/talent-factory/norma/commit/d5f63d5f840b2c0ae684751a4b00a741d145acf3))

- Seed the four MVP default patterns (no-debug-print, ticket 05) ([57e4891](https://github.com/talent-factory/norma/commit/57e4891653f93c838fa998ddd66555fb72edcdf4))

- Add SQLite PatternStore with fail-fast registration ([57694b5](https://github.com/talent-factory/norma/commit/57694b5e168914fba798b7b402a2644284d114ab))

- Add ast-grep-based pattern_engine::validate ([6dc5b2d](https://github.com/talent-factory/norma/commit/6dc5b2d5a583b0815f7815c0f7915d146eec79f9))

- Add Pattern/Severity/ValidationResult data model (ADR 0002) ([02cb4e9](https://github.com/talent-factory/norma/commit/02cb4e957e277c6e31de4f402d1681188811dd28))

- Switch dependencies to rmcp + ast-grep, add lib target ([e56f65d](https://github.com/talent-factory/norma/commit/e56f65d01ead54f9bd8ed7dd3ed930960338f518))

- Ignore .superpowers/ (SDD workspace scratch) ([a2b3b75](https://github.com/talent-factory/norma/commit/a2b3b75f4cca583bd1bb0e95a77fb5cb97ab5700))

- Add norma MVP implementation plan ([3038434](https://github.com/talent-factory/norma/commit/30384346cb264229c27f26d7a1505a1667b784a1))

- Resolve ticket: MVP-Pattern-Set-Scope ([7cb1be9](https://github.com/talent-factory/norma/commit/7cb1be9e335fa8d7c08181126693e2f08c0f4fef))

- Claim ticket: MVP-Pattern-Set-Scope (grilling) ([22aaf28](https://github.com/talent-factory/norma/commit/22aaf2812431c5df83c83989c1e7c288247f7221))

- Resolve ticket: Pattern-Datenmodell & SQLite-Schema ([d8bb355](https://github.com/talent-factory/norma/commit/d8bb3552c18012afde838b4a222a035b73e0cacb))

- Claim ticket: Pattern-Datenmodell & SQLite-Schema (grilling) ([aa06fe4](https://github.com/talent-factory/norma/commit/aa06fe4f16bf5ca4674f866aa8d94764e9b1828e))

- Resolve ticket: CLI/Pre-Commit-Architektur ([ac99edc](https://github.com/talent-factory/norma/commit/ac99edc7f86f68a78c1eabf4b8b8eec9255eaf25))

- Claim ticket: CLI/Pre-Commit-Architektur (grilling) ([727174d](https://github.com/talent-factory/norma/commit/727174d0a7ef63bc0ab57c28e6d747cf2d2ccf25))

- Resolve ticket: Pattern-Matching-Ansatz → ast-grep-core ([05be2e6](https://github.com/talent-factory/norma/commit/05be2e609dcdcb9d5c9b171e4c10262e0e393a27))

- Claim ticket: Pattern-Matching-Ansatz (grilling) ([8af6085](https://github.com/talent-factory/norma/commit/8af60855fb4187ecca34eb787e0b5dd10e4e0fc3))

- Mcpkit vs. rmcp — empfehle Wechsel zu rmcp ([2aa5e62](https://github.com/talent-factory/norma/commit/2aa5e62ff312d8aa2c37366b767cd0606c97adb8))

- Claim ticket: mcpkit vs. alternative (research) ([eea05cc](https://github.com/talent-factory/norma/commit/eea05cc2047128580021d169789258600c48441a))

- Initial scaffold: norma MVP + wayfinder architecture map ([1b0e5b3](https://github.com/talent-factory/norma/commit/1b0e5b371205bb3781073a335b1ae004372aaddb))


<!-- generated by git-cliff -->
