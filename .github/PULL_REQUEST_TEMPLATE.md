<!--
  Attach at least one A- and one C- label.
  See .opencode/kb/github/labels.md for the full taxonomy.
  Labels: A-Terrain / A-Compute / A-Render / A-Game / A-Agent / A-CI / A-Docs
           C-Bug / C-Feature / C-Code-Quality / C-Performance / C-Docs / C-Question / C-Chore
-->

## Summary

## Related issue
ref #

## Checklist
- [ ] Tests added/updated
- [ ] `cargo nextest run --workspace` passes
- [ ] `cargo clippy --workspace -- -A dead_code -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] kb/ files updated if behavior changed
- [ ] Shader changes: verified with `--release` actual run (compile ≠ render correct)
