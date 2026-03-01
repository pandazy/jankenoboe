## v2.5.1

### Search: Romaji Name Support for Shows

The `search show` command now supports searching by `name_romaji`, enabling lookups of shows by their Japanese romaji names. All match modes are supported: `exact`, `exact-i`, `starts-with`, `ends-with`, and `contains`.

```bash
# Find show by romaji name (case-insensitive)
jankenoboe search show --fields id,name,name_romaji --term '{"name_romaji":{"value":"yubisaki to renren","match":"exact-i"}}'

# Find shows whose romaji name contains "kimi"
jankenoboe search show --fields id,name,name_romaji,vintage --term '{"name_romaji":{"value":"kimi","match":"contains"}}'
```

### Documentation: Consolidate docs/ and skills/

Reduced duplication between `docs/cli-*.md` and `.claude/skills/` by establishing clear ownership:
- **`docs/cli-*.md`** — Developer technical reference (args, options, field tables, SQL, algorithms, error cases)
- **`.claude/skills/`** — User-facing guides (usage examples, workflows, output formats)

Each CLI doc page now links to the relevant skill(s) for examples. Removed redundant command examples and output samples from docs.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)