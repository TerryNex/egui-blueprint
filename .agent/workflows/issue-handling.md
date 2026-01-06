---
description: Standard workflow for handling user bug reports and feature requests
---

# Issue Handling Workflow

## When User Reports Issues/Requests Features

### Step 1: Create/Update TODO.md

// turbo

1. Check if `TODO.md` exists in project root
2. Create or update it with ALL reported issues
3. Use the following format for each item:

```markdown
### [Bug/Feature] #N: Short Title
- **Status:** `[ ]`
- **Issue:** Clear description
- **Root Cause:** Analysis if known
- **Solution:** Proposed implementation
- **Complexity:** Low/Medium/High
```

### Step 2: Categorize Items

Organize by priority:

- **High Priority - Bugs**: Blocking issues, data loss, crashes
- **High Priority - Features**: Core functionality requests
- **Medium Priority**: UI/UX improvements, non-critical features
- **Low Priority**: Nice-to-have, future enhancements
- **Cannot Fix / Deferred**: Document why

### Step 3: Implementation Phase

For each item being worked on:

1. Mark status as `[/]` In Progress
2. Implement the fix/feature
3. Test locally
4. Mark as `[x]` Completed when done

### Step 4: Final Verification

// turbo
Before delivering ANY code changes, run:

```bash
cargo check
```

Fix any compilation errors before submitting.

### Step 5: Documentation

- Update `README.md` for new features
- Update `CHANGELOG.md` with changes
- Update `agent.md` if project context changes

## Key Rules

1. **NEVER lose issue information** - Keep unresolved items in TODO.md
2. **Always run cargo check** before final delivery
3. **Update status markers** as work progresses
