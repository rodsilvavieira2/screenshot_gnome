---
name: git-commit
description: Create standardized git commit messages based strictly on staged changes.
license: MIT
metadata:
  audience: developers
  workflow: git
---

# Git Commit Skill

## What I do
I help generate descriptive, consistent, and well-formatted git commit messages. I analyze **only the currently staged changes** (the index) to ensure the commit message accurately reflects what is actually being committed, ignoring unstaged modifications.

## When to use me
Use this skill whenever you need to commit changes to the repository. Specifically:
- When you have already run `git add`.
- When you want to ensure the commit message follows [Conventional Commits](https://www.conventionalcommits.org/) standards.

## How to use me
1. **Check Staged Status:** I will first check if there are files staged for commit. If nothing is staged, I will ask you to stage files first.
2. **Analyze Index:** I will read the diff of the staged files *only* (using `git diff --cached` or equivalent). I **must not** consider changes in the working directory that have not been added to the index.
3. **Draft:** I will draft a commit message using the format: `<type>(<scope>): <description>`.
4. **Confirm:** I will propose the message to you before executing `git commit -m "..."`.

### Formatting Rules
- **Types:** Use `feat`, `fix`, `docs`, `style`, `refactor`, `test`, or `chore`.
- **Scope:** Provide a brief scope (e.g., `auth`, `ui`, `api`).
- **Description:** Keep it imperative, lowercase, and concise.
