# Copyright and licensing

Prism follows the aiaiaiai organization copyright policy and keeps its repository-specific Apache-2.0 license.

## Identity

- public copyright owner: `aiaiaiai`;
- canonical origin: `aiaiaiai.org`;
- canonical signature: `© 2026 aiaiaiai · aiaiaiai.org`;
- `aiaiaiai-org` is the GitHub repository location, not the copyright owner;
- this repository does not claim a registered legal entity.

The first year stays the year of initial copyrightable authorship. Extend the year range only for later material authorship, not formatting, lockfile regeneration, or automated dependency updates.

## Repository license

The repository license is Apache-2.0. Authored Rust, TOML, and YAML files use the canonical signature followed by:

```text
SPDX-License-Identifier: Apache-2.0
```

`LICENSE` contains the full license text. `NOTICE` contains the repository identity and distribution notice. Copyright identifies authorship; SPDX identifies the granted license.

## File treatment

| Material | Treatment |
| --- | --- |
| Authored Rust, TOML, and YAML | Copyright + SPDX header at the top |
| Authored Markdown | Non-rendering canonical footer |
| Generated JSON Schema | No injected comment |
| JSON fixtures | No comments |
| Lockfiles and build artifacts | No manually maintained header |
| Vendored or derived third-party material | Preserve upstream notices and attribution |

`cargo run -p xtask -- check` validates generated contracts and repository copyright rules. `cargo run -p xtask -- check-copyright` runs only the copyright/license checks.

## Future license changes

This document does not establish dual, multi, proprietary, or commercial licensing. Any such change requires an explicit repository decision and must not silently replace Apache-2.0.

If compound licensing is approved later, source headers must use the exact approved SPDX expression. Per-file overrides and required third-party notices take precedence.

Policy precedence is applicable law, repository-specific LICENSE/NOTICE rules, then the aiaiaiai organization default.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
