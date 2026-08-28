import io

p = 'tools/ui-verify/src/checks/form_field.rs'
s = io.open(p, encoding='utf-8').read()

# --- 1. lift the scroll loop into a helper ---------------------------------
old = '''    let mut required = None;
    for attempt in 0..SCROLL_ATTEMPTS {
        let trace = session.trace()?;
        if let Some(rect) = driving::declared(&trace, ui_rect, REQUIRED_REGION) {
            required = Some(rect);
            if attempt > 0 {
                report.note(format!(
                    "the editable properties were below the panel's fold; {attempt} scroll \\
                     notch(es) brought them into view"
                ));
            }
            break;
        }
        let Some(anchor) = driving::declared(&trace, ui_rect, PROPERTIES_REGION) else {
            return Err(Error::new(format!(
                "the form-field section stopped being visible while scrolling for its editable \\
                 properties, so there is nothing left to aim at. Trace: {}.",
                session.trace_path().display()
            )));
        };
        driver.scroll_at(session.frame()?.declared_center(anchor), -1)?;
        session.settle(12);
    }

    let trace = session.trace()?;
    let required = required else {'''
new = '''    let required = scroll_to(
        &session,
        &driver,
        ui_rect,
        PROPERTIES_REGION,
        REQUIRED_REGION,
        report,
    )?;

    let trace = session.trace()?;
    let Some(required) = required else {'''
# the original had `let Some(required) = required else {` — normalise both forms
if old in s:
    s = s.replace(old, new, 1)
else:
    old = old.replace('    let required = required else {', '    let Some(required) = required else {')
    assert old in s, 'scroll loop anchor'
    s = s.replace(old, new, 1)

# --- 2. the helper ----------------------------------------------------------
old2 = '''#[allow(clippy::too_many_lines)]
fn drive('''
new2 = '''/// **Scroll the Properties panel until `wanted` is on screen, and answer where
/// it is.**
///
/// # ★★★ Why this is a helper and not two copies of a loop
///
/// It was two copies for about ten minutes, and the second copy is what forced
/// the extraction: the field-scoped controls sit below the fold of the
/// Properties slot, and the widget-scoped controls sit below *those*. A check
/// that scrolled once found the first and reported the second missing — which
/// is the failure this function's existence prevents, and it is worth naming
/// because the message it produced was confident and wrong (*"the section is
/// not being called"*, about a section that was on screen).
///
/// ★ It scrolls **at an anchor the application already published**, never at a
/// guessed point. `anchor` is a region known to be visible — the enclosing
/// section — so the wheel event lands inside the scroll area rather than over
/// the canvas or another panel. `D:/dev/rag/egui/` carries the general form:
/// harness coordinates go stale when a layout changes, and a wheel aimed at a
/// remembered position scrolls whatever is there now.
///
/// Returns `None` when the region never appears — the caller decides whether
/// that is a failure or a skip, because only the caller knows what the region
/// means.
fn scroll_to(
    session: &Session,
    driver: &Driver,
    ui_rect: &str,
    anchor: &str,
    wanted: &str,
    report: &mut CheckReport,
) -> Result<Option<crate::coords::LRect>> {
    for attempt in 0..SCROLL_ATTEMPTS {
        let trace = session.trace()?;
        if let Some(rect) = driving::declared(&trace, ui_rect, wanted) {
            if attempt > 0 {
                report.note(format!(
                    "`{wanted}` was below the panel's fold; {attempt} scroll notch(es) brought \\
                     it into view"
                ));
            }
            return Ok(Some(rect));
        }
        let Some(at) = driving::declared(&trace, ui_rect, anchor) else {
            return Err(Error::new(format!(
                "`{anchor}` stopped being visible while scrolling for `{wanted}`, so there is \\
                 nothing left to aim the wheel at. Trace: {}.",
                session.trace_path().display()
            )));
        };
        driver.scroll_at(session.frame()?.declared_center(at), -1)?;
        session.settle(12);
    }
    Ok(None)
}

#[allow(clippy::too_many_lines)]
fn drive('''
assert old2 in s
s = s.replace(old2, new2, 1)

# --- 3. the widget half scrolls too ----------------------------------------
old3 = '''    let trace = session.trace()?;
    let Some(spinner) = driving::declared(&trace, ui_rect, WIDGET_X_REGION) else {'''
new3 = '''    // ★ Scroll again. The widget-scoped controls sit below the field-scoped
    // ones, which were themselves below the fold — so one scroll reaches the
    // first set and not the second, and the first run of this step reported the
    // box controls missing while `properties.widget_edit` was in the very same
    // trace. The anchor is that section's own rect, which is why it is
    // published with the ungated `ui_rect`.
    let spinner = scroll_to(
        &session,
        &driver,
        ui_rect,
        WIDGET_SECTION_REGION,
        WIDGET_X_REGION,
        report,
    )?;
    let trace = session.trace()?;
    let Some(spinner) = spinner else {'''
assert old3 in s
s = s.replace(old3, new3, 1)

old4 = '''// ★ `properties.widget_edit` — the section's own rect — is deliberately NOT a
// constant here and deliberately not asserted. `fieldedit`'s twin taught the
// lesson an hour earlier: a section rect is a scroll anchor and a yes/no
// answer, and the controls inside it are what a check clicks. Naming it would
// invite a second assertion that adds nothing and can go stale on a publishing
// convention rather than on the feature.
'''
new4 = '''/// The widget-scoped section's own rect.
///
/// ★★ Used as a **scroll anchor** and deliberately never asserted on. A section
/// rect answers *"did this draw?"* and *"where do I aim the wheel?"*; the
/// controls inside it are what a check clicks. `fieldedit`'s twin taught that
/// an hour earlier, when a check failed on a section rect that was absent for a
/// publishing-convention reason while the feature underneath worked.
const WIDGET_SECTION_REGION: &str = "properties.widget_edit";
'''
assert old4 in s
s = s.replace(old4, new4, 1)
io.open(p, 'w', encoding='utf-8').write(s)
print('ok')
