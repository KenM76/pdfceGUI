# `fixtures/forms-xobject` — where these two files came from, and why they are here

Two single-page PDFs, 200 × 200 pt each, that put drawn content **inside a form
XObject** (ISO 32000-1 §8.10.1) rather than straight onto the page.

## Provenance

**Copied verbatim, byte for byte, from `D:\Dev\pdfce\fixtures\synthetic\forms-xobject\`.**

They are wholly synthetic — authored for the pdfce project by its committed
generator `tools/gen-form-recursion-fixtures.py`, from object syntax read off
ISO 32000-1. Nothing derives from a third-party file, so no attribution is owed
or claimed and nothing was downloaded. That is `pdfce/docs/LEGAL.md` §5
category (a), and it is the reason a copy is unproblematic.

They are assembled from a classic §7.5.4 cross-reference table, with no object
streams and no compression on the structure, so a failing test can be diagnosed
in a hex editor.

## ★ Why copied rather than referenced across the repository boundary

`D:\Dev\pdfce` is **read-only to this project until fold-in day**, and that is
the governing rule of the whole rebuild — but read-only is not the reason. The
reason is that a unit test which reaches out of its own workspace has a
dependency nothing declares: it passes on this machine, fails on a machine that
has only this repository checked out, and fails in a way whose message is about
a missing file rather than about the thing under test.

`tools/ui-verify` does reach into the engine's fixtures in two places, and that
is a different bargain: it is an operator-run harness on this machine, driving a
binary built against that engine. A `cargo test` must not need two repositories.

At fold-in these two files become duplicates of the engine's own and should be
deleted in favour of them — noted here so that whoever does the fold-in finds
the note rather than the puzzle.

## The two files

| file | shape | what it pins |
|---|---|---|
| `page-sized-form.pdf` | one page-sized form holding **three** separate squares | the operator's case exactly: **1** page object, **3** leaves. The form's `/BBox` is the whole `MediaBox`, so before the deep hit test it won every click at every point |
| `nested-forms.pdf` | form A holds form B holds one square | containment depth **2**, and that an intermediate form is **not** itself a leaf |

`page-sized-form.pdf`'s three squares are at `10,10`–`50,50`, `80,80`–`120,120`
and `150,150`–`190,190` in PDF user space, filled blue. The gaps between them
matter as much as the squares: a click in one is a click *inside the form* that
must select **nothing**, which is the assertion that stops a "fall back to the
form when the deep query is empty" fix from ever being reintroduced.

## The engine's own note on why the directory had to exist at all

> No committed fixture in this repository had a form XObject. Every vector
> fixture draws straight onto the page, so the entire form branch of the
> decomposer was exercised only by a stub resolver that returns a *shape* and no
> *content* — which is precisely the branch that turned out to be missing.
>
> The operator's report, relayed from the GUI project: *"when I click on one of
> the objects all I get is the page selected."* He was selecting a real object.
> It was a page-sized form, sitting in paint order above everything drawn before
> it, answering every click anywhere on the sheet.
