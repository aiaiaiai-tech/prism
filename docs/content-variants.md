# Content variants

Prism publishes an intent, not one canonical text.

One publication may contain several explicit `ContentVariant` values. Each
variant describes how the same intent should appear in one context. Prism does
not infer one dimension from another.

| Dimension | Meaning | Example |
| --- | --- | --- |
| `locale` | Language and locale | `uk-UA` |
| `voice_profile` | How the identity speaks | `0x0sky.uk_SP` |
| `audience` | Who the variant is for | Ukraine, Kyiv, donors |
| `provider_target` | Optional provider-specific variant | `meta.instagram` |
| `format` | Publication surface | `post`, `story` |
| `body` | Text and media | caption + image |
| `provenance` | Where the variant came from | human, import, transform, AI |

## Rules

- Prism has no canonical language. A Ukrainian variant is not implicitly
  derived from an English one, or vice versa.
- Locale is not voice. `uk-UA` is a locale; `uk_SP` is a style/voice profile.
- Voice, audience, provider, and format are independent choices.
- Translation, localization, stylization, or AI generation happen before
  delivery. Their output must be an explicit `ContentVariant`.
- Provider adaptation is separate from voice. A Threads post and an Instagram
  story may use different variants even with the same locale and voice.
- Fallback is explicit. A target selects one exact variant or an ordered list;
  Prism does not silently fall back to another language or style.
- Generated variants keep provenance. AI does not bypass normal validation or
  delivery rules.

For example, one release announcement may have:

- `uk-UA` + `0x0sky.uk_SP` + Threads post;
- `en-GB` + `0x0sky.en` + Threads post;
- `uk-UA` + `0x0sky.uk_SP` + Instagram story.

These are peer representations of one publication. None must be the source text
for the others.

In short: **one intent → explicit variants → deterministic delivery**.

<!-- © 2026 aiaiaiai · aiaiaiai.org -->
