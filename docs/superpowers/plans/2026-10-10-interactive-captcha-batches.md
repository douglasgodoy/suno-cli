# Interactive CAPTCHA and batch session plan

User-approved design: reduce repeated browser launches by keeping one isolated browser per CLI batch, use a single visible window for human CAPTCHA interaction, preserve preflight and fresh tokens, and add safe diagnostics. CAPTCHA requirements remain controlled by Suno.

- Add `--rounds 1..10` to generate/Cover, default 1; submissions remain sequential and each has its own stable request ID and durable receipt. Add `--request-id` to Cover for batch recovery.
- Reject a supplied single-use token for multi-round batches before authentication. Skip already-receipted rounds without solving another CAPTCHA. Include partial IDs and batch recovery context on failures.
- Preview every planned request offline for generate; Cover preview still reads its identified source. Preserve single-round output compatibility.
- Select exactly one provider from preflight. Normal mode uses one visible human-interactive window, headless remains invisible, and no-browser remains HTTP-only. No fallback cycles or token reuse.
- Reuse the browser/tab and initialize stored authentication once per process. Wait for the destination document before loading the selected SDK. Poll interactive completion with a bounded timeout, log stages without secrets, and clean up Chrome on exit/Ctrl-C.
- Verify red/green contracts, full suite, formatting, clippy, independent review and release build. No extra music generation during development; resume the previously requested remix only after validation and with human CAPTCHA interaction if required.

Review focus: partial batch failures must preserve IDs; reruns must not duplicate completed rounds; explicit token flags cannot cause token reuse; no-browser/headless must honor their promises; one-round callers keep existing envelope shape; manual mode must expose the challenge and never click/solve it automatically.

Validation: 204 Rust tests and 6 JavaScript widget tests passed; formatting, diff whitespace and clippy with warnings denied passed. Independent review found document reuse could select another origin after human navigation; fixed origin filtering and per-round readiness validation, then re-reviewed with no blocking findings. HTTP recovery regression confirms two saved rounds only perform feed GETs. Live provider acceptance still requires the music test.
