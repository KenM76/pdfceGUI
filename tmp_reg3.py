# -*- coding: utf-8 -*-
import io

p = 'EDITABLE_SURFACES.md'
s = io.open(p, encoding='utf-8').read()

old = """**157 `EditSession` verbs. 144 named somewhere in the shell. 13 named nowhere.**
At the start of the audit it was 135 / 22."""
assert s.count(old) == 1
new = """**157 `EditSession` verbs. 147 named somewhere in the shell. 10 named nowhere.**
At the start of the audit it was 135 / 22.

★★★ **Measured against the LOCKED revision, not the engine's working tree**, and
the distinction earned itself within a day. The first cut of the tool read
`edit.rs` off disk and reported `move_outline_item` and `set_outline_open` as
gaps — bookmark reorder, re-parent and open state, which the engine's own note
had said did *not* ship. They were **uncommitted work in the engine session's
worktree**, which that project edits continuously while this one runs.

⇒ A verb in the worktree and not in the lock **is not callable from here**, and
a register listing it would send the next session to write a call that does not
compile while looking like a capability we were behind on. The tool now prints
those under `COMING` and keeps the two facts apart: *"nothing here calls it"*
and *"we could not call it if we wanted to."* Both are worth knowing; they are
not the same thing."""
s = s.replace(old, new)

# the preview/refusal section gets its outcome
old = """⇒ Each is R9 and R83 quality work rather than a missing capability: the verb
runs either way, and the difference is whether the operator learns the answer
before the gesture or from a refusal after it."""
assert s.count(old) == 1
new = """⇒ Each is R9 and R83 quality work rather than a missing capability: the verb
runs either way, and the difference is whether the operator learns the answer
before the gesture or from a refusal after it.

**Three of the four were built on 2026-08-29. `paste_preview` was declined**,
and the decline is recorded rather than left as an omission: `edit.paste`'s own
registration already carries a decision against a greying Paste — *"a control
that greys and un-greys under the pointer is harder to aim at than one that
answers in a sentence when pressed"* — and the engine's case for the verb is
quoted from the *requesting* shell, which wanted the greyed menu item this one
decided not to have. The cost is also real and per-frame: the clip is a
`Vec<u8>` in `egui::Memory` that `read()` clones out, so the condition would
clone it and re-parse an `ObjectClip` on every ribbon frame.

★ The gap next door is the one worth having instead: `vector_edit`'s `Err` arm
is **silent for every verb**, which is what made the annotation delete a silence
in the first place. Wording it is a placement decision, not a consumer for this
query."""
s = s.replace(old, new)
io.open(p, 'w', encoding='utf-8', newline='').write(s)
print('ok')
