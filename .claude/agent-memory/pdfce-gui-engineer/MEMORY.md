# Memory index

- [ui-verify competes for the machine](feedback_ui_verify_competes_for_the_machine.md) — the harness drives the real desktop; needs Ken's go-ahead, and never softens R1.
- [Update the engine before every build](feedback_update_engine_before_every_build.md) — `cargo update` core/render/print first; the engine repo moves fast and a stale pin has already cost 18 missing images.
- [The pdfce specialist agents are not on this roster](project_pdfce_agent_roster_absent_here.md) — dispatching them fails here; do it inline or use general-purpose.
- [Scope a request to the whole expected behaviour](feedback_scope_a_request_to_the_whole_expected_behaviour.md) — Ken expects what surrounds a request too; enumerating deferrals just moves the work onto him.
- [The engine session runs in parallel](project_the_engine_session_runs_in_parallel_and_answers_within_the_hour.md) — it answers requests within minutes and dirties the read-only tree; that is not a violation.
- [Always publish the latest build to OneDrive](feedback_always_publish_the_latest_build_to_onedrive.md) — `package-portable.py` after every keeper build; it alternates pdfceGUI1/2 itself, so the previous one survives.
- [Refresh FEATURES.md before every release](feedback_refresh_features_md_before_every_release.md) — re-measure against the build, then package; he reads it to know what he has.
- [Never defer on an external blocker](feedback_never_defer_on_an_external_blocker.md) — decompose the operation into verbs that exist; three "blockers" were never real.
- [Use the conventional interaction, never invent one](feedback_use_the_conventional_interaction_never_invent_one.md) — the convergence of the product class IS the spec; an invented model is a defect even when it works.
- [pdfce is multi-document since 2026-08-20](project_pdfce_is_multi_document_since_2026_08_20.md) — the active doc is still `PdfceApp::status`; don't modernise it into `documents[active]`.
- [Smoke-launch offscreen when the desktop is blocked](feedback_smoke_launch_offscreen_when_the_desktop_is_blocked.md) — `PDFCE_DIAG_VIEWPORT` proves a surface is drawn without touching the pointer.
- [Requests live in a file, not a conversation](feedback_operator_requests_live_in_a_file_not_a_conversation.md) — every ask goes in OPERATOR_REQUESTS.md at once; only Ken closes a row.
- [A guard that stops repetition does not stop creep](feedback_a_guard_that_stops_repetition_does_not_stop_creep.md) — a measurement fed back into a size needs a direction bound and a floor, not a "don't ask twice".
- [A measurement of the wrong surface looks exactly like a broken one](feedback_a_measurement_of_the_wrong_surface_looks_exactly_like_a_broken_one.md) — ask what a failing pixel check SAMPLED before asking what is broken.
- [Disk is tight and target/ grows unbounded](project_disk_is_tight_and_target_grows_unbounded.md) — 50GB+ of stale cache a week; clear debug/doc routinely, never release.
- [A backlog row is a record, not evidence](feedback_a_backlog_row_is_a_record_not_evidence.md) — verify absence claims against source; three docs said the rotate grip was missing a day after it shipped.
- [A check that cannot fail is not evidence](feedback_a_check_that_cannot_fail_is_not_evidence.md) — falsify before quoting green; make it SKIP when it never saw the mechanism.
- [Ken's sentences are reports, not measurements](feedback_kens_sentences_are_reports_not_measurements.md) — "up to 800%" named an old setting, not a threshold; measure the boundary he names.
