---
name: pdfce-agent-roster-absent-here
description: The pdfce specialist subagents are not in this project's agent roster — dispatching them fails; do the analysis inline or use general-purpose
metadata:
  type: project
---

**`pdfce-ui-specialist`, `pdfce-librarian`, `pdfce-spec-librarian` and
`pdfce-acrobat-librarian` are NOT available from a session running in
`D:\Dev\pdfceGUI`.** Dispatching one returns
`Agent type 'pdfce-ui-specialist' not found`, and the error lists the roster
actually present: `general-purpose`, `Explore`, `Plan`, `claude`,
`autonomous-builder`, `troubleshooting-*`, `tax-rag`, `claude-code-guide`, and
this role itself.

**Why:** observed directly on 2026-08-18, from a tool error rather than
inferred — those agent definitions live under `D:\Dev\pdfce\.claude\agents\`
and this project is a separate working directory. The `pdfce-gui-engineer`
definition names all four as dispatch targets, which was written when the role
was expected to run from the pdfce tree.

**How to apply:**

- Do not spend a turn discovering this. When the agent definition's dispatch
  table says to send work to one of those four, either do the analysis inline
  and *say in the report that the specialist was not reachable*, or dispatch
  `general-purpose` with the same prompt plus the reading list the specialist
  would have had.
- The **librarian check-in before compaction** cannot be performed here. The
  substitute that actually preserves the work is the one this project already
  uses: write findings to `D:/dev/rag/egui/`, `D:/dev/rag/rust/` and
  `C:\personal_rag\`, and re-measure `RESUME.md` — see
  [[feedback-ui-verify-competes-for-the-machine]] for the sibling rule about
  reporting unverified work honestly rather than softening it.
- This is worth re-checking rather than trusting forever: adding the four
  definitions under `D:\Dev\pdfceGUI\.claude\agents\` would fix it, and is a
  reasonable thing for a future session or the operator to do.
