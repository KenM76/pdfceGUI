p = 'crates/pdfce-gui/src/app/blank.rs'
s = open(p, encoding='utf-8').read()

old = """    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "new-document-sized w={:.2} h={:.2} change={change:?} bytes={}",
            rect.width(),
            rect.height(),
            bytes.len(),
        )
    });

    let doc = Document::from_bytes(bytes).map_err(|err| err.to_string())?;
    let pages = pdfce_core::page_tree::pages(&doc).map_err(|err| err.to_string())?;
    Ok((doc, pages))"""

new = """    let doc = Document::from_bytes(bytes).map_err(|err| err.to_string())?;
    let pages = pdfce_core::page_tree::pages(&doc).map_err(|err| err.to_string())?;

    // ★ Traced AFTER the re-parse, and reporting the page as the RE-PARSED
    // document states it rather than the rectangle that was asked for.
    //
    // The distinction is the whole value of the line. A trace of the request
    // says what this function was told; a trace of `pages[0].media_box` says
    // what a reader of the resulting file will see, which is what the operator
    // gets and the only thing worth asserting from outside the process.
    // `ui-verify`'s `new_document_sizes_the_page` reads `result_w`/`result_h`
    // for exactly that reason — a build that recorded the request and wrote
    // nothing would have a perfect `w=`/`h=` and a 595 × 842 page.
    let media = pages.first().map(|page| page.media_box);
    crate::diag::trace(|| {
        format!(
            // ui-text-exempt: diagnostic trace, never displayed in the UI
            "new-document-sized w={:.2} h={:.2} change={change:?} bytes={} \
             result_w={:.2} result_h={:.2}",
            rect.width(),
            rect.height(),
            bytes.len(),
            media.map_or(0.0, |m| m.urx - m.llx),
            media.map_or(0.0, |m| m.ury - m.lly),
        )
    });

    Ok((doc, pages))"""
assert s.count(old) == 1
s = s.replace(old, new, 1)
open(p, 'w', encoding='utf-8', newline='').write(s)

# --- publish the size-list entries and the orientation radios -----------------
p = 'crates/pdfce-gui/src/dialogs/new_document.rs'
s = open(p, encoding='utf-8').read()

old2 = """/// The Create button.
const REGION_CREATE: &str = "new-document.create";"""
new2 = """/// The Create button.
const REGION_CREATE: &str = "new-document.create";

/// One region per entry in the OPEN size list, indexed from zero in
/// `pdfce_core::paper::PaperSize::ALL` order, with `Custom…` last.
///
/// # Why the entries are published
///
/// The same argument the print dialog's paper list makes: an egui combo popup
/// is an `Area` laid out at paint time, so nothing outside the process can
/// compute where an entry is — and a check that can open a list but not choose
/// from it can assert only that a control exists. "The control exists" is
/// exactly what was true of the print dialog's tray checkbox for four months
/// while it did nothing.
///
/// Here the property worth asserting is that **picking a size produces a page
/// of that size**, end to end through `set_media_box`, a full rewrite and a
/// re-parse. That needs a click on a specific entry.
const REGION_SIZE_ITEM_PREFIX: &str = "new-document.size.item.";

/// The two orientation radios.
///
/// Published because the transposition is the most likely defect in this
/// window and the one a unit test cannot see end to end: `sheet_pt` is pinned
/// in tests, and what is *not* pinned there is that the radio the operator
/// clicks is the one that reaches it.
const REGION_PORTRAIT: &str = "new-document.portrait";
const REGION_LANDSCAPE: &str = "new-document.landscape";"""
assert s.count(old2) == 1
s = s.replace(old2, new2, 1)

old3 = """                for size in pdfce_core::paper::PaperSize::ALL {
                    ui.selectable_value(
                        &mut self.choice,
                        Choice::Standard(*size),
                        t::size_entry(&t::size_name(*size), size.size_pt()),
                    );
                }
                ui.selectable_value(&mut self.choice, Choice::Custom, t::size_custom());"""
new3 = """                for (index, size) in pdfce_core::paper::PaperSize::ALL.iter().enumerate() {
                    let entry = ui.selectable_value(
                        &mut self.choice,
                        Choice::Standard(*size),
                        t::size_entry(&t::size_name(*size), size.size_pt()),
                    );
                    crate::diag::ui_rect(
                        &format!("{REGION_SIZE_ITEM_PREFIX}{index}"),
                        entry.rect,
                    );
                }
                let custom = ui.selectable_value(&mut self.choice, Choice::Custom, t::size_custom());
                crate::diag::ui_rect(
                    &format!(
                        "{REGION_SIZE_ITEM_PREFIX}{}",
                        pdfce_core::paper::PaperSize::ALL.len()
                    ),
                    custom.rect,
                );"""
assert s.count(old3) == 1
s = s.replace(old3, new3, 1)

old4 = """        if ui.radio(!self.landscape, t::orientation_portrait()).clicked() {
            self.landscape = false;
        }
        if ui.radio(self.landscape, t::orientation_landscape()).clicked() {
            self.landscape = true;
        }"""
new4 = """        let portrait = ui.radio(!self.landscape, t::orientation_portrait());
        crate::diag::ui_rect(REGION_PORTRAIT, portrait.rect);
        if portrait.clicked() {
            self.landscape = false;
        }
        let landscape = ui.radio(self.landscape, t::orientation_landscape());
        crate::diag::ui_rect(REGION_LANDSCAPE, landscape.rect);
        if landscape.clicked() {
            self.landscape = true;
        }"""
assert s.count(old4) == 1
s = s.replace(old4, new4, 1)
open(p, 'w', encoding='utf-8', newline='').write(s)
print('ok')
