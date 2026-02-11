use std::ffi::OsString;

use x11rb::connection::Connection;
use x11rb::cursor::Handle as CursorHandle;
use x11rb::protocol::xfixes;
use x11rb::protocol::xinput;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::resource_manager::Database;
use x11rb::rust_connection::RustConnection;

use crate::backend::{DisplayBackend, MotionEvent};

/// X11 implementation of the display backend.
///
/// Connects to the X server via the DISPLAY environment variable,
/// subscribes to MotionNotify events on the root window, and handles
/// cursor size changes using Xcursor.
pub struct X11Backend {
    /// Connection to the X server. None until connect() is called.
    /// RustConnection is x11rb's pure Rust transport — no C libraries needed.
    conn: Option<RustConnection>,

    /// X11 screen number, needed when creating cursor handles.
    /// Received from the X server during connection setup.
    screen_num: usize,

    /// The root window ID. This is the top-level window that covers the
    /// entire screen. We subscribe to MotionNotify on this window to
    /// receive every mouse movement, regardless of which application
    /// has focus.
    root: u32,

    /// The original cursor size in pixels, read from the user's X resource
    /// database on startup. Used to restore the cursor after a shake ends.
    original_cursor_size: u32,

    /// The Xcursor theme name discovered on startup. Injected into the
    /// modified resource database so CursorHandle loads sized Xcursor
    /// files instead of falling back to fixed-size core X11 cursors.
    cursor_theme: String,
}

impl X11Backend {
    /// Create an unconnected backend. Call connect() to establish
    /// the X server connection.
    pub fn new() -> Self {
        Self {
            conn: None,
            screen_num: 0,
            root: 0,
            original_cursor_size: 24,
            cursor_theme: String::from("default"),
        }
    }
}

impl DisplayBackend for X11Backend {
    /// Observer pattern: connect to X server and subscribe to MotionNotify
    /// on the root window to receive every mouse movement event.
    fn connect(&mut self) -> Result<(), String> {
        let (conn, screen_num) = RustConnection::connect(None)
            .map_err(|err| format!("Failed to connect to X server: {}", err))?;

        let screen = &conn.setup().roots[screen_num];
        self.root = screen.root;
        self.screen_num = screen_num;

        // Negotiate XInput2 with the X server (minimum version 2.0).
        // XInput2 lets us receive motion events from the root window
        // regardless of which application window the pointer is over.
        // The core protocol's PointerMotion only works when the pointer
        // is directly on the root window background.
        xinput::xi_query_version(&conn, 2, 0)
            .map_err(|err| format!("Failed to query XInput2: {}", err))?
            .reply()
            .map_err(|err| format!("XInput2 not supported: {}", err))?;

        // Observer pattern: subscribe to XInput2 Motion events on root window
        // for all master pointer devices. This captures every mouse movement
        // across the entire screen, even over application windows.
        let event_mask = xinput::EventMask {
            deviceid: xinput::Device::ALL.into(),
            mask: vec![xinput::XIEventMask::MOTION],
        };
        xinput::xi_select_events(&conn, self.root, &[event_mask])
            .map_err(|err| format!("Failed to select XInput2 events: {}", err))?
            .check()
            .map_err(|err| format!("X server rejected event selection: {}", err))?;

        log::info!("XInput2 event selection accepted by server.");

        // Initialize XFixes extension for global cursor replacement.
        // XFixes lets us change the cursor across ALL windows at once,
        // not just the root window.
        xfixes::query_version(&conn, 6, 0)
            .map_err(|err| format!("Failed to query XFixes: {}", err))?
            .reply()
            .map_err(|err| format!("XFixes not supported: {}", err))?;

        conn.flush()
            .map_err(|err| format!("Failed to flush X connection: {}", err))?;

        // Read original cursor size from the X resource database.
        // This queries the RESOURCE_MANAGER property on the root window,
        // which contains settings like "Xcursor.size: 24".
        let rm_reply = conn.get_property(
            false,
            self.root,
            AtomEnum::RESOURCE_MANAGER,
            AtomEnum::STRING,
            0,
            1024 * 1024,
        )
        .map_err(|err| format!("Failed to query resource manager: {}", err))?
        .reply()
        .map_err(|err| format!("Failed to read resource manager reply: {}", err))?;

        let db = Database::new_from_default(&rm_reply, OsString::new());
        if let Some(size_str) = db.get_string("Xcursor.size", "Xcursor.Size") {
            if let Ok(size) = size_str.parse::<u32>() {
                self.original_cursor_size = size;
            }
        }

        // Discover the cursor theme name from the resource database.
        // If empty or unset, fall back to "default" which on most distros
        // inherits from Adwaita via /usr/share/icons/default/index.theme.
        if let Some(theme) = db.get_string("Xcursor.theme", "Xcursor.Theme") {
            if !theme.is_empty() {
                self.cursor_theme = theme.to_string();
            }
        }
        log::info!("Cursor theme: {}, original size: {}px", self.cursor_theme, self.original_cursor_size);

        self.conn = Some(conn);
        Ok(())
    }

    /// Block until the next mouse motion event arrives from the X server.
    /// Returns None if the connection is lost (Xorg crashed or was restarted).
    /// Non-motion events are silently skipped.
    fn next_motion_event(&mut self) -> Option<MotionEvent> {
        let conn = self.conn.as_ref()?;

        loop {
            match conn.wait_for_event() {
                Ok(Event::XinputMotion(motion)) => {
                    // XInput2 coordinates are Fp1616 (fixed-point 16.16).
                    // Right-shift by 16 to get integer pixel values.
                    return Some(MotionEvent {
                        x: (motion.root_x >> 16) as i16,
                        y: (motion.root_y >> 16) as i16,
                        timestamp: motion.time,
                    });
                }
                Ok(other) => {
                    log::debug!("Received non-motion event: {:?}", other);
                    continue;
                }
                Err(err) => {
                    log::error!("X connection error: {:?}", err);
                    return None;
                }
            }
        }
    }

    /// Load a cursor at the given size from the current theme and apply it
    /// to the root window. Used for both enlarging and restoring.
    fn set_cursor_size(&mut self, size: u32) -> Result<(), String> {
        let conn = self.conn.as_ref()
            .ok_or_else(|| "Not connected to X server".to_string())?;

        // Query the real resource database to get the cursor theme name,
        // then build a modified version with our desired cursor size.
        // CursorHandle reads size from the resource database, not from
        // the XCURSOR_SIZE env var, so we must override it here.
        let rm_reply = conn.get_property(
            false,
            self.root,
            AtomEnum::RESOURCE_MANAGER,
            AtomEnum::STRING,
            0,
            1024 * 1024,
        )
        .map_err(|err| format!("Failed to query resource manager: {}", err))?
        .reply()
        .map_err(|err| format!("Failed to read resource manager reply: {}", err))?;

        // Take the existing resource string, remove Xcursor settings we need
        // to override, then inject our desired values.
        // - Xcursor.size: the cursor size in pixels
        // - Xcursor.theme: force a named theme so CursorHandle loads Xcursor
        //   files instead of falling back to core X11 bitmap cursors
        // - Xcursor.theme_core: remove this flag because when set to 1 it
        //   forces CursorHandle to use fixed-size core cursors, ignoring
        //   the Xcursor size entirely
        let original = String::from_utf8_lossy(&rm_reply.value);
        let modified: String = original
            .lines()
            .filter(|line| {
                !line.starts_with("Xcursor.size")
                    && !line.starts_with("Xcursor.theme_core")
                    && !line.starts_with("Xcursor.theme")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let modified = format!(
            "{}\nXcursor.size: {}\nXcursor.theme: {}\n",
            modified, size, self.cursor_theme
        );

        // Build a new resource database with our modified size
        let modified_reply = GetPropertyReply {
            format: 8,
            sequence: 0,
            length: 0,
            type_: AtomEnum::STRING.into(),
            bytes_after: 0,
            value_len: modified.len() as u32,
            value: modified.into_bytes(),
        };
        let db = Database::new_from_default(&modified_reply, OsString::new());

        log::info!("Loading cursor at size {}px (theme: {})", size, self.cursor_theme);

        // CursorHandle::new returns a Cookie that must be resolved with .reply()
        let cursor_handle = CursorHandle::new(conn, self.screen_num, &db)
            .map_err(|err| format!("Failed to create cursor handle: {}", err))?
            .reply()
            .map_err(|err| format!("Failed to resolve cursor handle: {}", err))?;

        let cursor = cursor_handle.load_cursor(conn, "left_ptr")
            .map_err(|err| format!("Failed to load cursor: {}", err))?;

        log::debug!("Cursor XID: {} for size {}px", cursor, size);

        // XFixes: replace the cursor globally across ALL windows.
        // Unlike change_window_attributes (which only affects root window),
        // change_cursor_by_name replaces every instance of the named cursor
        // in every application window on screen.
        //
        // We replace multiple cursor names because different toolkits use
        // different names for the default arrow cursor:
        //   "left_ptr" — X11 core name, used by GTK and many legacy apps
        //   "default"  — freedesktop.org standard, used by Electron (VSCode), Qt
        for name in [b"left_ptr" as &[u8], b"default"] {
            xfixes::change_cursor_by_name(conn, cursor, name)
                .map_err(|err| format!("Failed to replace cursor '{}': {}",
                    String::from_utf8_lossy(name), err))?;
        }

        conn.flush()
            .map_err(|err| format!("Failed to flush X connection: {}", err))?;

        Ok(())
    }

    /// Close the connection to the X server.
    /// Dropping RustConnection automatically closes the underlying socket.
    fn disconnect(&mut self) {
        self.conn = None;
    }
}
