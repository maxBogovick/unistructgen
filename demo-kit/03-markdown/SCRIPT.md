# 🎬 Script: Video 3 - Markdown as Code

**Duration:** ~1.5 minutes
**Goal:** Show the "Single Source of Truth" concept.

---

### 0:00 - The "Sync" Problem
**(Screen: Show `CONFIG.md`)**
**You:** "Documentation usually lies. You write a README table, but the code does something else. Why not make the documentation the *source* of the code?"

### 0:30 - Parsing Markdown
**(Screen: Split view: `CONFIG.md` on left, Terminal on right)**
**You:** "Here is a config table in Markdown."
**(Action: Run command)**
```bash
unistructgen generate -i CONFIG.md -n AppConfig
```

### 0:50 - Verification
**(Screen: Show output struct)**
**You:** "UniStructGen parsed the table rows!
- `host` is String.
- `max_conn` is `Option<i32>` because the table said 'Required: no'.
- `allowed_ips` became `Vec<String>`."

### 1:10 - Live Update
**(Action: Edit the Markdown file. Change 'port' type from 'integer' to 'string'. Save.)**
**(Action: Re-run the command.)**
**You:** "Update the docs, regenerate the code. Your documentation is now always up to date."
