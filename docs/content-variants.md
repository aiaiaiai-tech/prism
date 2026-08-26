# Content variants

Prism publishes an intent, not one canonical text.

One publication may contain several explicit `ContentVariant` values. Each variant describes how the same intent should appear in one context. Prism never infers one dimension from another.

| Dimension | Meaning | Example |
| --- | --- | --- |
| `locale` | Language and locale | `uk-UA` |
| `voice_profile` | How the identity speaks | `0x0sky.uk_SP` |
| `audience` | Who the variant is for | Ukraine, Kyiv, donors |
| `provider_target` | Optional provider-specific variant | `meta.instagram` |
| `format` | Publication surface | `post`, `story` |
| `body` | Text and media | caption + image |
| `provenance` | Where the variant came from | human, import, transform, ai |

## Rules

- Prism has no canonical language. A Ukrainian variant is not implicitly derived from an English one, or vice versa.
- Locale is not voice. `uk-UA` is a locale; `uk_SP` is a voice profile.
- Voice profile, audience, provider target, and format are independent choices.
- Translation, localization, stylization, or ai generation happens before delivery and must produce an explicit `ContentVariant`.
- Provider adaptation is separate from voice. A Threads post and an Instagram story may use different variants with the same locale and voice profile.
- Fallback is explicit. A target selects one exact variant or an ordered list. Prism never silently changes language or voice profile.
- Generated variants keep provenance. ai never bypasses normal validation or delivery rules.

One release announcement may therefore have:

- `uk-UA` + `0x0sky.uk_SP` + Threads post;
- `en-GB` + `0x0sky.en` + Threads post;
- `uk-UA` + `0x0sky.uk_SP` + Instagram story.

These are peer representations of one publication. None has to be the source text for the others.

**one intent → explicit variants → deterministic delivery**

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
