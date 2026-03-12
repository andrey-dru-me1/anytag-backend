# Git Workflow Guide

This document outlines the Git workflow and conventions for the anytag-backend project. Following these guidelines ensures consistent collaboration, clean commit history, and efficient code review processes.

## Branch Strategy

We follow a simplified Git Flow approach with two main long-lived branches:

### Main Branches

1. **`master`** - Production-ready code
   - Always stable and deployable
   - Contains tagged releases
   - Protected branch (no direct pushes)

2. **`develop`** - Integration branch for features
   - Contains completed features ready for next release
   - Should be stable and pass all tests
   - Protected branch (no direct pushes)

### Supporting Branches

All development happens in feature branches branched from `develop`:

- **Feature branches** (`feature/*`) - New functionality
- **Bugfix branches** (`bugfix/*`) - Bug fixes for upcoming release
- **Hotfix branches** (`hotfix/*`) - Critical production fixes (branched from `master`)
- **Release branches** (`release/*`) - Release preparation (branched from `develop`)

## Branch Naming Conventions

Use lowercase with hyphens for separation:

```txt
feature/add-user-authentication
bugfix/fix-null-pointer-exception
hotfix/critical-security-patch
release/v1.2.0
```

## Commit Message Guidelines

### Format

```txt
<type>: <subject>

<body>

<footer>
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, missing semicolons, etc.)
- **refactor**: Code refactoring (no functional changes)
- **test**: Adding or updating tests
- **chore**: Maintenance tasks, dependencies, build scripts

### Subject Rules

- Use imperative mood: "Add" not "Adds" or "Added"
- Start with capital letter
- No period at the end
- Keep under 50 characters

### Examples

```txt
feat: Add user authentication middleware
fix: Resolve database connection timeout
docs: Update API endpoint documentation
refactor: Simplify error handling in handlers
```

### Body (Optional)

- Explain what and why, not how
- Wrap at 72 characters
- Use bullet points for multiple changes

### Footer (Optional)

- Reference issues: `Closes #123`, `Fixes #456`
- Breaking changes: `BREAKING CHANGE: <description>`

## Pull Request Workflow

### Creating a Pull Request

1. **Create feature branch** from `develop`:

   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/your-feature-name
   ```

2. **Make changes** and commit following conventions:

   ```bash
   git add .
   git commit -m "feat: Add new endpoint for user profiles"
   ```

3. **Push to remote**:

   ```bash
   git push -u origin feature/your-feature-name
   ```

4. **Create PR** on GitHub:
   - Target: `develop` branch
   - Use template if available
   - Link related issues

### PR Title Format

```txt
[Type] Brief description of changes
```

Examples:

```txt
[Feature] Add user authentication
[Bugfix] Resolve memory leak in database pool
[Refactor] Simplify error handling middleware
```

### PR Description Template

```markdown
## Description
Brief description of the changes.

## Changes Made
- Change 1
- Change 2
- Change 3

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Manual testing performed

## Related Issues
Closes #123
Fixes #456

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated if needed
- [ ] No breaking changes introduced
```

## Code Review Process

### For Reviewers

1. **Be specific**: Reference lines and suggest alternatives
2. **Check requirements**: Ensure PR meets acceptance criteria
3. **Test locally** if significant changes

### Review Checklist

- [ ] Code follows project conventions
- [ ] No security vulnerabilities introduced
- [ ] Tests are comprehensive and pass
- [ ] Documentation is updated if needed
- [ ] Performance considerations addressed
- [ ] Error handling is appropriate

### For Authors

1. **Address all feedback** before requesting re-review
2. **Resolve conversations** - The person who started a conversation should resolve it when the issue is addressed
3. **Keep PR focused** - one feature/bug per PR
4. **Rebase if needed** to resolve conflicts
5. **Squash commits** if requested

## Merging Strategy

### Squash and Merge

- **When to use**: Feature branches with multiple commits
- **Benefits**: Clean history, one commit per feature
- **Commit message**: Use PR title as squash commit message

### Merge Commit

- **When to use**: When preserving individual commit history is important
- **Benefits**: Full history preserved
- **Use case**: Complex features with meaningful intermediate commits

### Rebase and Merge

- **When to use**: Simple branches, linear history preferred
- **Benefits**: Linear history without merge commits

## Common Workflows

### Starting a New Feature

```bash
# Update local develop
git checkout develop
git pull origin develop

# Create feature branch
git checkout -b feature/feature-name

# Make changes and commit
git add .
git commit -m "feat: Add feature description"

# Push to remote
git push -u origin feature/feature-name
```

### Updating Feature Branch with Latest Develop

```bash
# From feature branch
git checkout feature/feature-name

# Fetch latest changes
git fetch origin

# Rebase onto develop
git rebase origin/develop

# Resolve conflicts if any
# Continue rebase
git rebase --continue

# Force push (since history changed)
git push -f
```

### Creating a Hotfix

```bash
# From master
git checkout master
git pull origin master

# Create hotfix branch
git checkout -b hotfix/issue-description

# Make urgent fix
git add .
git commit -m "fix: Critical security patch"

# Push and create PR to master AND develop
git push -u origin hotfix/issue-description
```

## Best Practices

### Do's

- ✅ Keep branches up-to-date with base branch
- ✅ Write descriptive commit messages
- ✅ Make small, focused PRs (300-500 lines max)
- ✅ Review your own code before requesting review
- ✅ Run tests before pushing
- ✅ Use `.gitignore` appropriately

### Don'ts

- ❌ Commit directly to `master` or `develop`
- ❌ Force push to shared branches
- ❌ Leave PRs open for extended periods without updates
- ❌ Merge your own PR without at least one review
- ❌ Commit large binary files
- ❌ Break the build

## Troubleshooting

### Common Issues

**Merge conflicts:**

```bash
# Update local branch
git fetch origin
git rebase origin/develop

# Resolve conflicts, then
git add .
git rebase --continue
```

**Accidental commit to wrong branch:**

```bash
# Create new branch with the commit
git branch feature/correct-branch

# Reset original branch
git checkout original-branch
git reset --hard HEAD~1
```

**Need to amend last commit:**

```bash
git commit --amend
# Edit message or add forgotten files
git push -f  # Only if not pushed yet or on your own branch
```

## Tools and Automation

### Git Hooks

Consider setting up pre-commit hooks for:

- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- Commit message validation
- Running tests

### GitHub Actions

Automated checks should include:

- Build status
- Test execution
- Code coverage
- Linting results

## Additional Resources

- [Conventional Commits](https://www.conventionalcommits.org/)
- [Git Flow](https://nvie.com/posts/a-successful-git-branching-model/)
- [GitHub Pull Request Best Practices](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests)

---

*This document is a living guide. Please suggest improvements via PRs.*
