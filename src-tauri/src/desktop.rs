//! OS-specific input stays here. No background clipboard polling or key logging.
use clipboard_rs::{Clipboard, ClipboardContent, ClipboardContext, ContentFormat};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::{
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    app: u64,
    window: u64,
    element: u64,
    range: Option<(isize, isize)>,
}

pub struct Selection {
    pub text: String,
    pub target: Target,
}

pub fn has_permissions() -> bool {
    native::has_permissions()
}
pub fn request_permissions() -> bool {
    native::request_permissions()
}
pub fn target_unchanged(target: &Target) -> bool {
    native::target().as_ref() == Ok(target)
}

pub fn set_clipboard_text(text: &str) -> Result<(), String> {
    ClipboardContext::new()
        .map_err(|_| "Clipboard unavailable.")?
        .set_text(text.into())
        .map_err(|_| "Could not copy the result.".into())
}

fn wait_for_modifiers() -> Result<(), String> {
    let started = Instant::now();
    while native::modifiers_down() {
        if started.elapsed() > Duration::from_secs(3) {
            return Err("Release the shortcut keys and try again.".into());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Ok(())
}

fn chord(key: char) -> Result<(), String> {
    let settings = Settings {
        open_prompt_to_get_permissions: false,
        ..Settings::default()
    };
    let mut keyboard = Enigo::new(&settings)
        .map_err(|_| "Allow KeyDock in System Settings → Privacy & Security → Accessibility.")?;
    let modifier = if cfg!(target_os = "macos") {
        Key::Meta
    } else {
        Key::Control
    };
    keyboard
        .key(modifier, Direction::Press)
        .map_err(|_| "Could not send keyboard shortcut.")?;
    let result = keyboard.key(Key::Unicode(key), Direction::Click);
    let released = keyboard.key(modifier, Direction::Release);
    result
        .and(released)
        .map_err(|_| "Could not send keyboard shortcut.".into())
}

/// Restores only while we still own this clipboard generation. A copy made by
/// the user while an API request is running is never overwritten.
struct ClipboardGuard {
    context: ClipboardContext,
    backup: Option<Vec<ClipboardContent>>,
    generation: u64,
}

impl ClipboardGuard {
    fn new() -> Result<Self, String> {
        let context = ClipboardContext::new().map_err(|_| "Clipboard unavailable.")?;
        let generation = native::clipboard_generation();
        let mut backup = Vec::new();
        // Common rich representations are preserved together. Clipboard-rs
        // cannot round-trip application-owned or multi-item formats exactly.
        let formats = [
            ContentFormat::Text,
            ContentFormat::Rtf,
            ContentFormat::Html,
            ContentFormat::Image,
            ContentFormat::Files,
        ];
        for format in formats {
            if context.has(format.clone()) {
                let mut contents = context.get(&[format]).map_err(|_| {
                    "Could not back up the clipboard. Copy plain text and try again."
                })?;
                backup.append(&mut contents);
            }
        }
        if generation != native::clipboard_generation() {
            return Err("Clipboard changed. Try again.".into());
        }
        if backup.is_empty()
            && !context
                .available_formats()
                .map_err(|_| "Clipboard unavailable.")?
                .is_empty()
        {
            return Err("The clipboard contains an unsupported format. Copy plain text first so it can be restored safely.".into());
        }
        Ok(Self {
            context,
            backup: Some(backup),
            generation,
        })
    }

    fn copy_selection(&mut self, target: &Target) -> Result<Option<String>, String> {
        if !target_unchanged(target) {
            return Ok(None);
        }
        let before = native::clipboard_generation();
        if before != self.generation {
            return Ok(None);
        }
        chord('c')?;
        let start = Instant::now();
        while native::clipboard_generation() == before {
            if !target_unchanged(target) || start.elapsed() > Duration::from_millis(900) {
                return Ok(None);
            }
            thread::sleep(Duration::from_millis(15));
        }
        self.generation = native::clipboard_generation();
        if !target_unchanged(target) {
            return Ok(None);
        }
        Ok(self
            .context
            .get_text()
            .ok()
            .filter(|s| !s.trim().is_empty()))
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        if self.generation != native::clipboard_generation() {
            return;
        }
        if let Some(contents) = self.backup.take() {
            // The Windows implementation clears the clipboard when setting
            // images; put the image first so text/HTML/RTF can follow it.
            let mut images = Vec::new();
            let mut others = Vec::new();
            for content in contents {
                if matches!(content, ClipboardContent::Image(_)) {
                    images.push(content);
                } else {
                    others.push(content);
                }
            }
            images.extend(others);
            let _ = if images.is_empty() {
                self.context.clear()
            } else {
                self.context.set(images)
            };
        }
    }
}

pub fn capture_selection(own_window_focused: bool) -> Result<Selection, String> {
    if own_window_focused {
        return Err("Select text in another app, then press your prompt shortcut.".into());
    }
    if !has_permissions() {
        return Err("Allow KeyDock in System Settings → Privacy & Security → Accessibility, then try again.".into());
    }
    wait_for_modifiers()?;
    let target = native::target()?;
    #[cfg(target_os = "macos")]
    if matches!(target.range, Some((_, 0))) {
        return Err("Select some text first.".into());
    }
    let mut guard = ClipboardGuard::new()?;
    let text = guard.copy_selection(&target)?.ok_or(
        "No selected text could be copied. Select text in an editable field and try again.",
    )?;
    if text.len() > 80_000 {
        return Err("Select a shorter text (up to 80,000 bytes).".into());
    }
    Ok(Selection { text, target })
}

pub fn replace_selection(selection: &Selection, output: &str) -> Result<bool, String> {
    wait_for_modifiers()?;
    if !target_unchanged(&selection.target) {
        return Ok(false);
    }
    let mut guard = ClipboardGuard::new()?;
    let current = guard.copy_selection(&selection.target)?;
    if !can_replace(
        &selection.text,
        current.as_deref(),
        target_unchanged(&selection.target),
    ) {
        return Ok(false);
    }
    if selection.text == output {
        return Ok(true);
    }
    if guard.generation != native::clipboard_generation() || !target_unchanged(&selection.target) {
        return Ok(false);
    }
    guard
        .context
        .set_text(output.to_string())
        .map_err(|_| "Could not prepare the result for pasting.")?;
    guard.generation = native::clipboard_generation();
    thread::sleep(Duration::from_millis(40));
    if !target_unchanged(&selection.target) {
        return Ok(false);
    }
    chord('v')?;
    thread::sleep(Duration::from_millis(400));
    Ok(true)
}

fn can_replace(original: &str, selected_now: Option<&str>, target_same: bool) -> bool {
    target_same && selected_now == Some(original)
}

#[cfg(target_os = "macos")]
mod native {
    use super::Target;
    use std::{
        ffi::{c_char, c_void},
        ptr,
    };
    type Ref = *const c_void;
    #[repr(C)]
    struct Range {
        location: isize,
        length: isize,
    }
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn AXIsProcessTrustedWithOptions(options: Ref) -> bool;
        fn AXUIElementCreateSystemWide() -> Ref;
        fn AXUIElementCopyAttributeValue(element: Ref, attribute: Ref, value: *mut Ref) -> i32;
        fn AXUIElementGetPid(element: Ref, pid: *mut i32) -> i32;
        fn AXValueGetValue(value: Ref, kind: u32, result: *mut c_void) -> bool;
        static kAXTrustedCheckOptionPrompt: Ref;
    }
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceKeyState(state: i32, key: u16) -> bool;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(value: Ref);
        fn CFHash(value: Ref) -> usize;
        fn CFEqual(a: Ref, b: Ref) -> bool;
        fn CFStringCreateWithCString(allocator: Ref, value: *const c_char, encoding: u32) -> Ref;
        fn CFDictionaryCreate(
            allocator: Ref,
            keys: *const Ref,
            values: *const Ref,
            count: isize,
            key_callbacks: Ref,
            value_callbacks: Ref,
        ) -> Ref;
        static kCFBooleanTrue: Ref;
    }
    struct Owned(Ref);
    impl Drop for Owned {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { CFRelease(self.0) }
            }
        }
    }
    fn string(value: &str) -> Owned {
        let string = std::ffi::CString::new(value).expect("static AX attribute");
        Owned(unsafe { CFStringCreateWithCString(ptr::null(), string.as_ptr(), 0x08000100) })
    }
    fn attribute(element: Ref, name: &str) -> Option<Owned> {
        let mut value = ptr::null();
        if unsafe { AXUIElementCopyAttributeValue(element, string(name).0, &mut value) } == 0
            && !value.is_null()
        {
            Some(Owned(value))
        } else {
            None
        }
    }
    pub fn has_permissions() -> bool {
        unsafe { AXIsProcessTrusted() }
    }
    pub fn request_permissions() -> bool {
        unsafe {
            let options = Owned(CFDictionaryCreate(
                ptr::null(),
                &kAXTrustedCheckOptionPrompt,
                &kCFBooleanTrue,
                1,
                ptr::null(),
                ptr::null(),
            ));
            AXIsProcessTrustedWithOptions(options.0)
        }
    }
    pub fn modifiers_down() -> bool {
        [54, 55, 56, 58, 59, 60, 61, 62]
            .iter()
            .any(|key| unsafe { CGEventSourceKeyState(1, *key) })
    }
    pub fn clipboard_generation() -> u64 {
        objc2_app_kit::NSPasteboard::generalPasteboard().changeCount() as u64
    }
    pub fn target() -> Result<Target, String> {
        let system = Owned(unsafe { AXUIElementCreateSystemWide() });
        let app =
            attribute(system.0, "AXFocusedApplication").ok_or("Cannot identify the active app.")?;
        let mut pid = 0;
        if unsafe { AXUIElementGetPid(app.0, &mut pid) } != 0 {
            return Err("Cannot identify the active app.".into());
        }
        if pid == std::process::id() as i32 {
            return Err("Select text in another app first.".into());
        }
        let window =
            attribute(app.0, "AXFocusedWindow").ok_or("Cannot identify the active window.")?;
        let element = attribute(system.0, "AXFocusedUIElement")
            .ok_or("Cannot identify the selected text field.")?;
        if let Some(subrole) = attribute(element.0, "AXSubrole") {
            if unsafe { CFEqual(subrole.0, string("AXSecureTextField").0) } {
                return Err("Password fields are excluded.".into());
            }
        }
        let range = attribute(element.0, "AXSelectedTextRange").and_then(|value| {
            let mut range = Range {
                location: 0,
                length: 0,
            };
            if unsafe { AXValueGetValue(value.0, 4, &mut range as *mut Range as *mut c_void) } {
                Some((range.location, range.length))
            } else {
                None
            }
        });
        Ok(Target {
            app: pid as u64,
            window: unsafe { CFHash(window.0) } as u64,
            element: unsafe { CFHash(element.0) } as u64,
            range,
        })
    }
}

#[cfg(windows)]
mod native {
    use super::Target;
    use windows_sys::Win32::{
        System::DataExchange::GetClipboardSequenceNumber,
        UI::{
            Input::KeyboardAndMouse::GetAsyncKeyState,
            WindowsAndMessaging::{
                GetForegroundWindow, GetGUIThreadInfo, GetWindowLongW, GetWindowThreadProcessId,
                GUITHREADINFO, GWL_STYLE,
            },
        },
    };
    pub fn has_permissions() -> bool {
        true
    }
    pub fn request_permissions() -> bool {
        true
    }
    pub fn modifiers_down() -> bool {
        [0x10, 0x11, 0x12, 0x5B, 0x5C]
            .iter()
            .any(|key| unsafe { GetAsyncKeyState(*key) } < 0)
    }
    pub fn clipboard_generation() -> u64 {
        unsafe { GetClipboardSequenceNumber() as u64 }
    }
    pub fn target() -> Result<Target, String> {
        unsafe {
            let window = GetForegroundWindow();
            if window.is_null() {
                return Err("Cannot identify the active window.".into());
            }
            let mut pid = 0;
            let thread = GetWindowThreadProcessId(window, &mut pid);
            if pid == std::process::id() {
                return Err("Select text in another app first.".into());
            }
            let mut info: GUITHREADINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;
            if GetGUIThreadInfo(thread, &mut info) == 0 {
                return Err("Cannot identify the selected text field.".into());
            }
            // Native password edit controls use ES_PASSWORD. Browser password
            // fields also normally refuse copying and are rejected by timeout.
            if !info.hwndFocus.is_null() && GetWindowLongW(info.hwndFocus, GWL_STYLE) & 0x20 != 0 {
                let mut class = [0u16; 32];
                windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW(
                    info.hwndFocus,
                    class.as_mut_ptr(),
                    class.len() as i32,
                );
                if String::from_utf16_lossy(&class)
                    .trim_matches('\0')
                    .eq_ignore_ascii_case("Edit")
                {
                    return Err("Password fields are excluded.".into());
                }
            }
            let caret = if info.hwndCaret.is_null() {
                None
            } else {
                Some((info.rcCaret.left as isize, info.rcCaret.top as isize))
            };
            Ok(Target {
                app: pid as u64,
                window: window as usize as u64,
                element: info.hwndFocus as usize as u64,
                range: caret,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changed_focus_or_selection_never_allows_replacement() {
        assert!(!can_replace("original", Some("original"), false));
        assert!(!can_replace("original", Some("changed"), true));
        assert!(!can_replace("original", None, true));
        assert!(can_replace("original", Some("original"), true));
    }
}
