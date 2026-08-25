# Copyright and licensing

Prism follows the aiaiaiai organization copyright policy while retaining its
repository-specific Apache-2.0 license.

## Identity

- public copyright owner: `aiaiaiai`;
- canonical origin: `aiaiaiai.org`;
- canonical signature: `© 2026 aiaiaiai · aiaiaiai.org`;
- GitHub organization `aiaiaiai-org` is a repository location, not the
  copyright owner;
- no registered legal entity is claimed by this repository.

The first year remains the year of initial copyrightable authorship. A later
year extends the range only when a file receives material copyrightable
changes. Mechanical formatting, lockfile regeneration, and automated dependency
updates do not extend it by themselves.

## Current repository license

The repository-selected license is Apache-2.0. Authored Rust, TOML, and YAML
files use the canonical signature followed by:

```text
SPDX-License-Identifier: Apache-2.0
```

`LICENSE` contains the complete license text. `NOTICE` contains the repository
identity and concise distribution notice. Copyright and licensing remain
separate: the signature identifies authorship; the SPDX expression identifies
the granted license.

## File treatment

| Material | Treatment |
| --- | --- |
| Authored Rust, TOML, and YAML | Copyright and SPDX header at the top |
| Authored Markdown | Non-rendering canonical footer |
| Generated JSON Schema | No injected comment; generator output remains deterministic |
| JSON fixtures | No comments because standard JSON does not support them |
| Lockfiles and build artifacts | No manually maintained header |
| Vendored or derived third-party material | Preserve upstream notices and attribution |

`cargo run -p xtask -- check` validates generated contracts and the current
repository copyright policy. `cargo run -p xtask -- check-copyright` runs only
the copyright/license portion.

## Future license decisions

No dual, multi, proprietary, or commercial license is established by this
document. Such a change requires an explicit repository-level decision and
must not silently replace Apache-2.0.

If a future decision introduces compound licensing, source headers will use the
exact approved SPDX expression, including `OR`, `AND`, `WITH`, or a defined
`LicenseRef-*` where applicable. Per-file overrides and third-party notices take
precedence for the files they govern.

Policy precedence is applicable law, then repository-specific LICENSE/NOTICE
rules, then the aiaiaiai organization default.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
