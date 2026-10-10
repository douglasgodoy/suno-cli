# Cover controls implementation plan

**Goal:** permit the reggae/remix skill's Cover settings through the CLI without dropping source lyrics or source identity.

**Design:** adapt the title-resolution intent from upstream PR #6 to the current v2 request schema; add title, exclude, vocal, weirdness and style-influence arguments. Fetch source metadata once before CAPTCHA. Preserve source lyrics, source tags when not overridden, and instrumental status; preserve vocal direction unless explicitly requested. Build the real request in a pure function so serialized payloads can be tested without credentials or credits. Reuse existing slider normalization and API submission/receipt handling.

**Scope:** Cover only. No new authentication logic, no changes to Remaster/Extend, no automatic music generation or publication during development. Duration, Variety and Personalize remain outside this change because their API semantics have not been validated.

**Execution:** native implementation in this chat; tests first.

- [x] Run contract tests demonstrating that Cover lacks the new flags: three failures on unchanged upstream.
- [x] Add payload tests for source inheritance, explicit overrides, all slider values, instrumental sources and blank titles; test explicit voice conflicts with both genders and empty styles.
- [x] Implement parser arguments and request construction; preserve source lookup errors before submission.
- [x] Document exact flags and attribute the PR #6 title fix.
- [x] Run formatting, focused tests, full suite and clippy: 197 tests pass; fmt and clippy clean.
- [x] Build release binary: `cargo build --locked --release` completed successfully.
- [x] Independent read-only review completed; its vocal-direction finding was reproduced by a failing test, fixed and reviewed again.
- [x] Authenticated read-only Cover preview against an existing library source: source ID and nonempty lyrics retained, title/exclusions match, sliders are 0.2/0.7/0.7, submitted=false and no CAPTCHA token. No generation or credits consumed by the preview.
- [x] Authenticated missing-source preview: all-zero UUID returns invalid_input / exit 3 before CAPTCHA or submission.
- [x] Push branch/PR to the user's fork with validation evidence: https://github.com/douglasgodoy/suno-cli/pull/1.

**Review focus:** omitted flags must preserve source data; zero and 100 are valid while non-finite/out-of-range sliders are rejected; source ID remains bound to the Cover request; source-read failure must not turn into a new song; wire values must be normalized once, and no metadata should claim live musical validation.

**Limits:** actual Cover generation and musical fidelity remain unverified. The earlier composition test was blocked by provider-2 CAPTCHA. These changes do not fix CAPTCHA handling, and the existing installed CLI has not been replaced.
