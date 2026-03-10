# Task: Consolidate .clinerules into AGENTS.md

**Date:** 2026-03-02

## Summary

Merged `.clinerules` into `AGENTS.md` to eliminate redundancy and clarify role separation between operator-facing (`README.md`) and developer-facing (`AGENTS.md`) documentation.

## Changes

### Deleted
- `.clinerules` — all content absorbed into `AGENTS.md`

### AGENTS.md
- Replaced duplicated project intro with a one-line reference to README
- Added Module Responsibilities table (from .clinerules)
- Enriched JankenSQLHub Integration with full details + link to [JankenSQLHub usage skill](https://github.com/pandazy/jankensqlhub/blob/main/.claude/skills/using-jankensqlhub/SKILL.md)
- Added Routines section ("check README.md first")
- Added TODO List Usage subsection under Task Management
- Replaced outdated database setup paragraph with brief summary linking to initialize skill
- Replaced duplicated Agent Skills table with reference to README
- Removed `.clinerules` from document consistency checklist
- Changed "Non-technical users" → "Users"

### README.md
- Changed "Non-technical users" → "Users"

### docs/design/v1/development.md
- Removed `.clinerules` from Documentation Updates checklist

## Role Separation

| File | Audience | Content |
|------|----------|---------|
| `README.md` | Operators / users | Project intro, installation, setup, CLI usage, agent skills |
| `AGENTS.md` | Developers / AI agents | Build/test, architecture, conventions, domain concepts, JankenSQLHub |