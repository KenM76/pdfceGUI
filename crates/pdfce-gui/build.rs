//! # `build.rs` — put the application icon inside the executable
//!
//! One job. The operator asked, on 2026-08-18, for *"a pdf icon to the exe so
//! it shows as the icon when I associate it with pdfs"*, and an icon Explorer
//! can show is an icon in the executable's `.rsrc` section. Nothing loaded at
//! run time can satisfy that: the shell reads the icon **without running the
//! program**.
//!
//! `assets/pdfce-gui.rc` is the resource script and carries the reasoning for
//! what is in it — the icon's ID, and why the `VERSIONINFO` block is worth
//! having. This file is only the plumbing.
//!
//! ## Why `embed-resource` rather than invoking `rc.exe` here
//!
//! Because finding a resource compiler is the entire problem. `rc.exe` lives
//! inside a versioned Windows SDK directory that is not on `PATH`, its location
//! differs per SDK version and per machine, a cross build wants `windres`
//! instead, and the MSVC and GNU toolchains want the output linked differently.
//! `embed-resource` is a build-dependency that exists to know all of that, and
//! re-deriving it here would be a machine-specific path in a repository.
//!
//! It is a **build** dependency: it runs on this machine and nothing from it is
//! linked into `pdfce-gui.exe`. That is why it does not appear in
//! `THIRD_PARTY_LICENSES.md`, which `cargo-about` generates from the crates the
//! binary actually carries.
//!
//! ## Why `manifest_optional` and not `unwrap`
//!
//! An icon is **cosmetic**. A machine with no Windows SDK — a container, a
//! fresh CI image, a cross build — must still produce a working
//! `pdfce-gui.exe`, and failing the build over a missing resource compiler
//! would trade a program that opens PDFs for a program that does not exist.
//! `manifest_optional()` is `embed-resource`'s own name for exactly that
//! trade-off, and it is the one the crate's README recommends *"if the manifest
//! is cosmetic (like an icon)"*.
//!
//! The consequence is stated rather than hidden: on such a machine the build
//! succeeds and the executable has no icon. The `cargo:warning` below is what
//! says so, because a silent absence here looks identical to this file never
//! having been written.
//!
//! ## Why the `cfg` is on the CALL and not on the file
//!
//! A `build.rs` runs on the **host**, so `cfg!(windows)` here is a fact about
//! the machine doing the building. Gating the file itself is not possible —
//! Cargo compiles it either way — and gating the *dependency* in `Cargo.toml`
//! by target would be wrong for the same reason: `[build-dependencies]` are
//! host dependencies, and `[target.'cfg(windows)'.build-dependencies]` selects
//! on the **target**, so a Linux-hosted cross build to Windows would ask for a
//! crate it had not been given.

fn main() {
    // Re-run when either half of the resource changes. Without these, editing
    // the `.rc` or regenerating the `.ico` leaves the previous resource in the
    // executable and the change looks like it did nothing — which is a
    // particularly confusing failure for an icon, because Explorer caches them
    // too and the reader ends up blaming the wrong cache.
    println!("cargo:rerun-if-changed=assets/pdfce-gui.rc");
    println!("cargo:rerun-if-changed=assets/pdfce-gui.ico");

    #[cfg(windows)]
    {
        // `NONE` is `embed-resource`'s spelling of "no preprocessor
        // definitions"; the `.rc` needs none.
        let result = embed_resource::compile("assets/pdfce-gui.rc", embed_resource::NONE);
        if let Err(error) = result.manifest_optional() {
            // A warning, not a failure. See the header: the alternative is
            // trading a working program for a missing one over an icon.
            println!(
                "cargo:warning=pdfce-gui: the application icon was not embedded ({error}). The \
                 executable will build and run; Explorer will show it with the default icon. A \
                 Windows SDK (for rc.exe) is what supplies the resource compiler."
            );
        }
    }
}
