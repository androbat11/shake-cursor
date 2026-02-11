//! Diagnostic tests for X11 cursor loading.
//!
//! These tests connect to the running X server to verify that loading
//! cursors at different sizes actually produces different X11 resources.
//! Run with: cargo test --test x11_cursor_diagnostic -- --nocapture

use std::ffi::OsString;

use x11rb::connection::Connection;
use x11rb::cursor::Handle as CursorHandle;
use x11rb::protocol::xfixes;
use x11rb::protocol::xproto::*;
use x11rb::resource_manager::Database;
use x11rb::rust_connection::RustConnection;

/// Build a modified X resource database string by stripping existing
/// Xcursor settings and injecting the desired size and theme.
///
/// This is the same logic used by X11Backend::set_cursor_size.
fn build_cursor_resource_string(original: &str, size: u32, theme: &str) -> String {
    let filtered: String = original
        .lines()
        .filter(|line| {
            !line.starts_with("Xcursor.size")
                && !line.starts_with("Xcursor.theme_core")
                && !line.starts_with("Xcursor.theme")
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{}\nXcursor.size: {}\nXcursor.theme: {}\n",
        filtered, size, theme
    )
}

/// Build an x11rb Database from a raw resource string.
fn db_from_string(resource_string: &str) -> Database {
    let reply = GetPropertyReply {
        format: 8,
        sequence: 0,
        length: 0,
        type_: AtomEnum::STRING.into(),
        bytes_after: 0,
        value_len: resource_string.len() as u32,
        value: resource_string.as_bytes().to_vec(),
    };
    Database::new_from_default(&reply, OsString::new())
}

// ── Pure logic: resource database string manipulation ────────────

#[test]
fn resource_string_strips_xcursor_settings_and_injects_new_values() {
    let original = "\
Xft.dpi:\t96
Xcursor.size:\t0
Xcursor.theme:\t
Xcursor.theme_core:\t1
Xft.antialias:\t1";

    let modified = build_cursor_resource_string(original, 96, "default");

    // The three Xcursor lines should be removed
    assert!(!modified.contains("Xcursor.theme_core"));
    assert!(!modified.contains("Xcursor.size:\t0"));
    assert!(!modified.contains("Xcursor.theme:\t\n"));

    // Our injected values should be present
    assert!(modified.contains("Xcursor.size: 96"));
    assert!(modified.contains("Xcursor.theme: default"));

    // Unrelated settings should survive
    assert!(modified.contains("Xft.dpi:\t96"));
    assert!(modified.contains("Xft.antialias:\t1"));
}

#[test]
fn resource_string_handles_empty_input() {
    let modified = build_cursor_resource_string("", 48, "Adwaita");

    assert!(modified.contains("Xcursor.size: 48"));
    assert!(modified.contains("Xcursor.theme: Adwaita"));
}

#[test]
fn resource_db_parses_injected_size_correctly() {
    let modified = build_cursor_resource_string("", 96, "default");
    let db = db_from_string(&modified);

    let size_str = db.get_string("Xcursor.size", "Xcursor.Size")
        .expect("Xcursor.size should be present in the database");
    assert_eq!(size_str, "96");
}

#[test]
fn resource_db_parses_injected_theme_correctly() {
    let modified = build_cursor_resource_string("", 96, "default");
    let db = db_from_string(&modified);

    let theme = db.get_string("Xcursor.theme", "Xcursor.Theme")
        .expect("Xcursor.theme should be present in the database");
    assert_eq!(theme, "default");
}

#[test]
fn resource_db_has_no_theme_core_after_stripping() {
    let original = "Xcursor.theme_core:\t1\n";
    let modified = build_cursor_resource_string(original, 96, "default");
    let db = db_from_string(&modified);

    // theme_core should be gone — get_string should return None
    let theme_core = db.get_string("Xcursor.theme_core", "Xcursor.Theme_core");
    assert!(theme_core.is_none(), "Xcursor.theme_core should have been stripped");
}

// ── X11 integration: cursor loading at different sizes ───────────
//
// These tests require a running X server. They are marked #[ignore]
// so they don't run in CI. Run them explicitly with:
//   cargo test --test x11_cursor_diagnostic -- --ignored --nocapture

#[test]
#[ignore]
fn cursor_xids_differ_between_sizes() {
    let (conn, screen_num) = RustConnection::connect(None)
        .expect("Failed to connect to X server");

    let root = conn.setup().roots[screen_num].root;

    // Read the real resource database from the X server
    let rm_reply = conn.get_property(
        false,
        root,
        AtomEnum::RESOURCE_MANAGER,
        AtomEnum::STRING,
        0,
        1024 * 1024,
    )
    .expect("Failed to send get_property")
    .reply()
    .expect("Failed to read RESOURCE_MANAGER");

    let original = String::from_utf8_lossy(&rm_reply.value);
    println!("--- Original RESOURCE_MANAGER ---");
    for line in original.lines() {
        if line.contains("cursor") || line.contains("Cursor") || line.contains("Xcursor") {
            println!("  {}", line);
        }
    }

    // Load cursor at 24px
    let modified_24 = build_cursor_resource_string(&original, 24, "default");
    let db_24 = db_from_string(&modified_24);
    let handle_24 = CursorHandle::new(&conn, screen_num, &db_24)
        .expect("Failed to create cursor handle (24px)")
        .reply()
        .expect("Failed to resolve cursor handle (24px)");
    let cursor_24 = handle_24.load_cursor(&conn, "left_ptr")
        .expect("Failed to load cursor (24px)");

    // Load cursor at 96px
    let modified_96 = build_cursor_resource_string(&original, 96, "default");
    let db_96 = db_from_string(&modified_96);
    let handle_96 = CursorHandle::new(&conn, screen_num, &db_96)
        .expect("Failed to create cursor handle (96px)")
        .reply()
        .expect("Failed to resolve cursor handle (96px)");
    let cursor_96 = handle_96.load_cursor(&conn, "left_ptr")
        .expect("Failed to load cursor (96px)");

    println!("--- Cursor XIDs ---");
    println!("  24px: {}", cursor_24);
    println!("  96px: {}", cursor_96);
    println!("  Different: {}", cursor_24 != cursor_96);

    assert_ne!(
        cursor_24, cursor_96,
        "Cursor XIDs should differ between 24px and 96px. \
         If they are the same, CursorHandle is not respecting the size override."
    );
}

/// Visual test: applies a 96px cursor via XFixes, waits 3 seconds,
/// then restores. Run this and watch your cursor — it should get big.
///
/// Run with: cargo test --test x11_cursor_diagnostic visual_xfixes -- --ignored --nocapture
#[test]
#[ignore]
fn visual_xfixes_cursor_replacement() {
    use std::thread;
    use std::time::Duration;

    let (conn, screen_num) = RustConnection::connect(None)
        .expect("Failed to connect to X server");

    let root = conn.setup().roots[screen_num].root;

    // Initialize XFixes
    xfixes::query_version(&conn, 6, 0)
        .expect("Failed to query XFixes")
        .reply()
        .expect("XFixes not supported");

    let rm_reply = conn.get_property(
        false,
        root,
        AtomEnum::RESOURCE_MANAGER,
        AtomEnum::STRING,
        0,
        1024 * 1024,
    )
    .expect("Failed to send get_property")
    .reply()
    .expect("Failed to read RESOURCE_MANAGER");

    let original = String::from_utf8_lossy(&rm_reply.value);

    // Load 96px cursor
    let modified = build_cursor_resource_string(&original, 96, "default");
    let db = db_from_string(&modified);
    let handle = CursorHandle::new(&conn, screen_num, &db)
        .expect("Failed to create cursor handle")
        .reply()
        .expect("Failed to resolve cursor handle");
    let big_cursor = handle.load_cursor(&conn, "left_ptr")
        .expect("Failed to load 96px cursor");

    // Load 24px cursor for restoring
    let modified_restore = build_cursor_resource_string(&original, 24, "default");
    let db_restore = db_from_string(&modified_restore);
    let handle_restore = CursorHandle::new(&conn, screen_num, &db_restore)
        .expect("Failed to create restore cursor handle")
        .reply()
        .expect("Failed to resolve restore cursor handle");
    let small_cursor = handle_restore.load_cursor(&conn, "left_ptr")
        .expect("Failed to load 24px cursor");

    println!("Applying 96px cursor via XFixes (left_ptr + default)...");
    println!(">>> Look at your cursor — it should be BIG for 3 seconds <<<");

    // Replace both cursor names
    for name in [b"left_ptr" as &[u8], b"default"] {
        xfixes::change_cursor_by_name(&conn, big_cursor, name)
            .expect("Failed to replace cursor");
    }
    conn.flush().expect("Failed to flush");

    thread::sleep(Duration::from_secs(3));

    println!("Restoring 24px cursor...");
    for name in [b"left_ptr" as &[u8], b"default"] {
        xfixes::change_cursor_by_name(&conn, small_cursor, name)
            .expect("Failed to restore cursor");
    }
    conn.flush().expect("Failed to flush");

    println!("Done. Did the cursor change size?");
}
