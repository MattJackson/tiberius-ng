<!--
Thanks for contributing to tiberius-ng!

Please fill out the sections below. Keep the PR focused — smaller, single-purpose
pull requests are reviewed and merged faster.
-->

## Summary

<!-- What does this PR do, and why? -->

## Related issues

<!-- e.g. "Closes #123", "Refs #456". If there is no related issue, say so. -->

## Type of change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that changes existing public API)
- [ ] Documentation / tooling / CI only
- [ ] Refactor (no functional change)

## Checklist

- [ ] This PR targets the **`dev`** branch (feature work lands on `dev`, is promoted to `qa`, then `main`).
- [ ] `cargo fmt --all` has been run and the code is formatted.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes with no warnings.
- [ ] `cargo test` passes. Integration tests against a live SQL Server (via `TIBERIUS_TEST_CONNECTION_STRING` / `docker compose`) were run if the change affects runtime behaviour.
- [ ] Supply-chain check passes: `cargo deny check` is green (advisories, licenses, bans, sources).
- [ ] `CHANGELOG.md` has been updated with a user-facing entry (unless this change is not user-visible).
- [ ] Public API changes are documented (rustdoc), and new features are gated behind the appropriate Cargo feature flag where applicable.
- [ ] Commits are signed off / attributed to me, and I agree to license my contribution under the project's terms (MIT OR Apache-2.0).

## Testing

<!--
Describe how you verified the change. Include which async runtime / TLS backend
and which SQL Server version(s) you tested against, plus the feature flags used.
-->

## Additional notes

<!-- Anything reviewers should pay special attention to, follow-up work, etc. -->
