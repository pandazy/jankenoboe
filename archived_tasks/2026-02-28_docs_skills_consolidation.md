# Task: Consolidate docs/ and .claude/skills/ Documentation

**Date:** 2026-02-28
**Version:** 2.5.1

## Objective

Reduce duplicated information between `docs/` and `.claude/skills/` by establishing clear ownership boundaries.

## Problem

The `docs/cli-*.md` files and `.claude/skills/` SKILL.md files both contained comprehensive usage examples, output format samples, and step-by-step workflows. This created maintenance burden — any change to a command's behavior required updates in both locations, leading to drift.

## Strategy

- **`docs/cli-*.md`** = Developer technical reference: arguments/options tables, field whitelists, SQL queries, JankenSQLHub configs, algorithms, behavior notes, error cases
- **`.claude/skills/`** = User-facing guides: comprehensive usage examples, workflows, output formats, step-by-step instructions

Each doc page links to the relevant skill(s) for examples.

## Changes

| File | What changed |
|------|-------------|
| `docs/cli.md` | Added "Documentation Structure" section explaining the split |
| `docs/cli-querying.md` | Removed duplicated examples (exact/fuzzy search, output samples); kept arg tables, match mode reference, JankenSQLHub query configs, searchable column tables; added link to querying skill |
| `docs/cli-learning.md` | Removed duplicated command examples and output samples; kept SQL, algorithms, level-up path table, error cases, behavior notes; added links to learning + reviewing skills |
| `docs/cli-data-management.md` | Removed duplicated per-table create/update examples; kept field tables, behavior notes, URL encoding reference; added link to maintaining-data skill |

## Files Modified

| File | Change |
|------|--------|
| `docs/cli.md` | Added documentation structure section |
| `docs/cli-querying.md` | Slimmed down, linked to querying skill |
| `docs/cli-learning.md` | Slimmed down, linked to learning + reviewing skills |
| `docs/cli-data-management.md` | Slimmed down, linked to maintaining-data skill |