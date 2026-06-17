# Plans Summary

This document summarizes the two plans created for the codemod_recipe project and explains how to reference them in future sessions.

---

## 📋 Available Plans

### 1. **Option A Implementation Plan** (Todo List + File)
- **Purpose:** Split binaries into CLI and VS Code entry points
- **Format:** Interactive todo list + Markdown file (persists across sessions)
- **Todo Location:** Built into the system via `todo` tool
- **File Location:** `doc/OPTION_A_PLAN.md`
- **Status:** 18 tasks defined, all pending

**How to access:**
```bash
# Via todo tool (interactive)
todo read

# Or via file
read file_path="doc/OPTION_A_PLAN.md"
```

**Task IDs:** `option-a-1.1` through `option-a-4.2`

**Phases:**
- Phase 1: Create new CLI entry point (Tasks 1.1-1.7)
- Phase 2: Refactor codemod_host.dart (Tasks 2.1-2.4)
- Phase 3: Testing & Documentation (Tasks 3.1-3.5)
- Phase 4: Cleanup (Tasks 4.1-4.2)

---

### 2. **Code Quality Plan** (Stored File + Skill)
- **Purpose:** Guidelines and best practices for maintainable code
- **Format:** Markdown file + Vibe skill
- **File Location:** `doc/CODE_QUALITY_PLAN.md`
- **Skill Location:** `skills/code_quality/SKILL.md`

**How to access the file:**
```bash
read file_path="doc/CODE_QUALITY_PLAN.md"
```

**How to load the skill:**
```bash
skill name="code_quality"
```

**What the skill provides:**
- SOLID principles with Dart examples
- DRY and KISS guidelines
- Dart-specific best practices
- Code review checklist
- Common patterns (Factory, Builder, Strategy)
- Anti-patterns to avoid
- Tooling commands

---

## 🎯 How to Use in Future Sessions

### For Option A Implementation

1. **Start your session:**
   ```bash
   # View all todos
   todo read
   ```

2. **Work on a task:**
   ```bash
   # Mark task as in progress
   todo write todos=[
     {"id": "option-a-1.1", "content": "Create bin/codemod.dart with basic structure", "status": "in_progress", "priority": "high"},
     # Include all other todos with their current status
   ]
   ```

3. **Complete a task:**
   ```bash
   # Mark task as completed
   todo write todos=[
     {"id": "option-a-1.1", "content": "Create bin/codemod.dart with basic structure", "status": "completed", "priority": "high"},
     # Include all other todos
   ]
   ```

### For Code Quality

1. **Load the skill for guidance:**
   ```bash
   skill name="code_quality"
   ```

2. **Reference the detailed plan:**
   ```bash
   read file_path="doc/CODE_QUALITY_PLAN.md"
   ```

3. **Use the code review checklist:**
   - The skill includes a checklist for reviewing code
   - Use it during PR reviews or before committing

---

## 📁 File Locations

| Item | Location | Type | Persists? |
|------|----------|------|-----------|
| Option A Plan | System todo tool | Interactive | ✅ Yes |
| Code Quality Plan | `doc/CODE_QUALITY_PLAN.md` | File | ✅ Yes |
| Code Quality Skill | `skills/code_quality/SKILL.md` | Vibe Skill | ✅ Yes |

---

## 🚀 Recommended Starting Points

### If you want to start with Option A:
1. Run `todo read` to see all tasks
2. Start with `option-a-1.1`: Create `bin/codemod.dart`
3. Work through Phase 1 tasks sequentially

### If you want to start with Code Quality:
1. Load the skill: `skill name="code_quality"`
2. Review the SOLID principles section
3. Pick a high-priority task from `doc/CODE_QUALITY_PLAN.md`
4. Start with `Q-H-1`: Create `lib/src/constants.dart`

---

## 📊 Quick Reference

### Check Progress
```bash
# See what's done and what's pending
todo read
```

### View Detailed Plan
```bash
# Code Quality Plan
read file_path="doc/CODE_QUALITY_PLAN.md"

# This summary
read file_path="PLANS_SUMMARY.md"
```

### Load Skill for Guidance
```bash
skill name="code_quality"
```

---

## 🔄 Version Information

- **Option A Plan:** Created 2026-06-17, 18 tasks
- **Code Quality Plan:** Created 2026-06-17, 30+ tasks
- **Code Quality Skill:** Created 2026-06-17, enhanced with research

---

## 💡 Tips for Future Sessions

1. **Start with `todo read`** to see what's pending
2. **Load relevant skills** at the start of each session
3. **Update todo status** as you work
4. **Reference plan files** for detailed task descriptions
5. **Commit progress** regularly to avoid losing work

---

## 📞 Need Help?

If you forget how to access any plan, just ask:
- "Show me my Option A tasks"
- "Load the code quality skill"
- "What should I work on next?"
