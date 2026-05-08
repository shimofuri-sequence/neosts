use crate::settings::AppLocale;

#[derive(Debug, thiserror::Error)]
pub enum SendToAfterEffectsError {
    #[error("After Effects script execution is only supported on Windows and macOS")]
    UnsupportedPlatform,
    #[error("After Effects window was not found")]
    WindowNotFound,
    #[error("failed to resolve After Effects executable path")]
    ExecutablePathUnavailable,
    #[error("failed to write temporary JSX file")]
    WriteScript(#[source] std::io::Error),
    #[error("failed to launch After Effects")]
    Launch(#[source] std::io::Error),
}

impl SendToAfterEffectsError {
    pub fn localized_message(&self, locale: AppLocale) -> String {
        match self {
            Self::UnsupportedPlatform => match locale {
                AppLocale::Japanese => {
                    "After Effects スクリプト実行は Windows / macOS のみ対応です".to_owned()
                }
                AppLocale::English => {
                    "After Effects script execution is only supported on Windows and macOS"
                        .to_owned()
                }
            },
            Self::WindowNotFound => match locale {
                AppLocale::Japanese => "After Effects のウィンドウが見つかりません".to_owned(),
                AppLocale::English => "After Effects window was not found".to_owned(),
            },
            Self::ExecutablePathUnavailable => match locale {
                AppLocale::Japanese => {
                    "After Effects の実行ファイルを見つけられませんでした".to_owned()
                }
                AppLocale::English => {
                    "Could not resolve the After Effects executable path".to_owned()
                }
            },
            Self::WriteScript(source) => match locale {
                AppLocale::Japanese => {
                    format!("一時 JSX ファイルの書き込みに失敗しました: {source}")
                }
                AppLocale::English => {
                    format!("Failed to write temporary JSX file: {source}")
                }
            },
            Self::Launch(source) => match locale {
                AppLocale::Japanese => format!("After Effects の起動に失敗しました: {source}"),
                AppLocale::English => format!("Failed to launch After Effects: {source}"),
            },
        }
    }
}

#[cfg(target_os = "windows")]
pub fn after_effects_is_available(owner_hwnd: Option<isize>) -> bool {
    resolve_after_effects_executable(owner_hwnd).is_ok()
}

#[cfg(target_os = "windows")]
pub(crate) fn launch_after_effects_script(
    script_path: &std::path::Path,
    owner_hwnd: Option<isize>,
) -> Result<(), SendToAfterEffectsError> {
    use std::process::Command;

    let afterfx = resolve_after_effects_executable(owner_hwnd)?;
    Command::new(afterfx)
        .arg("-r")
        .arg(script_path)
        .spawn()
        .map_err(SendToAfterEffectsError::Launch)?;
    Ok(())
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub fn after_effects_is_available(_owner_hwnd: Option<isize>) -> bool {
    false
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub(crate) fn launch_after_effects_script(
    _script_path: &std::path::Path,
    _owner_hwnd: Option<isize>,
) -> Result<(), SendToAfterEffectsError> {
    Err(SendToAfterEffectsError::UnsupportedPlatform)
}

#[cfg(target_os = "macos")]
pub fn after_effects_is_available(_owner_hwnd: Option<isize>) -> bool {
    most_recent_ae_pid_from_window_order().is_some()
}

#[cfg(target_os = "macos")]
pub(crate) fn launch_after_effects_script(
    script_path: &std::path::Path,
    _owner_hwnd: Option<isize>,
) -> Result<(), SendToAfterEffectsError> {
    use objc2_foundation::{NSAppleEventDescriptor, NSAppleEventSendOptions, NSString};
    use std::path::Path;
    use std::process::Command;

    const fn four_char_code(code: [u8; 4]) -> u32 {
        ((code[0] as u32) << 24)
            | ((code[1] as u32) << 16)
            | ((code[2] as u32) << 8)
            | (code[3] as u32)
    }

    fn javascript_single_quoted_literal(value: &str) -> String {
        let mut escaped = String::new();
        for ch in value.chars() {
            match ch {
                '\\' => escaped.push_str("\\\\"),
                '\'' => escaped.push_str("\\'"),
                '\r' => escaped.push_str("\\r"),
                '\n' => escaped.push_str("\\n"),
                '\t' => escaped.push_str("\\t"),
                ch if ch.is_ascii() && !ch.is_control() => escaped.push(ch),
                ch => {
                    let mut buf = [0u16; 2];
                    for unit in ch.encode_utf16(&mut buf).iter().copied() {
                        escaped.push_str(&format!("\\u{unit:04X}"));
                    }
                }
            }
        }
        escaped
    }

    fn send_script_to_pid(pid: i32, script_path: &Path) -> Result<(), SendToAfterEffectsError> {
        use objc2_core_services::{
            AEEventClass, AEEventID, AEKeyword, AEReturnID, AETransactionID,
        };

        const EVENT_CLASS_MISC: u32 = four_char_code(*b"misc");
        const EVENT_ID_DOSC: u32 = four_char_code(*b"dosc");
        const KEY_DIRECT_OBJECT: u32 = four_char_code(*b"----");
        const AUTO_GENERATE_RETURN_ID: i16 = -1;
        const ANY_TRANSACTION_ID: i32 = 0;

        let script_path = script_path
            .to_str()
            .ok_or(SendToAfterEffectsError::ExecutablePathUnavailable)?;
        let eval_source = format!(
            "$.evalFile('{}')",
            javascript_single_quoted_literal(script_path)
        );
        let source = NSString::from_str(&eval_source);
        let source_descriptor = NSAppleEventDescriptor::descriptorWithString(&source);
        let target_descriptor = NSAppleEventDescriptor::descriptorWithProcessIdentifier(pid);
        let event = NSAppleEventDescriptor::appleEventWithEventClass_eventID_targetDescriptor_returnID_transactionID(
            EVENT_CLASS_MISC as AEEventClass,
            EVENT_ID_DOSC as AEEventID,
            Some(&target_descriptor),
            AUTO_GENERATE_RETURN_ID as AEReturnID,
            ANY_TRANSACTION_ID as AETransactionID,
        );
        event.setParamDescriptor_forKeyword(&source_descriptor, KEY_DIRECT_OBJECT as AEKeyword);
        event
            .sendEventWithOptions_timeout_error(NSAppleEventSendOptions::DefaultOptions, 120.0)
            .map_err(|error| {
                SendToAfterEffectsError::Launch(std::io::Error::other(
                    error.localizedDescription().to_string(),
                ))
            })?;
        Ok(())
    }

    fn installed_after_effects_name() -> Option<String> {
        installed_after_effects_app_path()?
            .file_stem()?
            .to_str()
            .map(str::to_owned)
    }

    if let Some(pid) = most_recent_ae_pid_from_window_order() {
        return send_script_to_pid(pid, script_path);
    }

    let application_name =
        installed_after_effects_name().ok_or(SendToAfterEffectsError::ExecutablePathUnavailable)?;
    let script_posix = script_path
        .to_str()
        .ok_or(SendToAfterEffectsError::ExecutablePathUnavailable)?;
    let output = Command::new("osascript")
        .args([
            "-e",
            &format!(r#"set jsxFile to POSIX file "{}""#, script_posix),
            "-e",
            &format!(r#"tell application "{}""#, application_name),
            "-e",
            "activate",
            "-e",
            "DoScriptFile jsxFile",
            "-e",
            "end tell",
        ])
        .output()
        .map_err(SendToAfterEffectsError::Launch)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(SendToAfterEffectsError::Launch(std::io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        )))
    }
}

#[cfg(target_os = "windows")]
fn resolve_after_effects_executable(
    owner_hwnd: Option<isize>,
) -> Result<std::path::PathBuf, SendToAfterEffectsError> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GW_HWNDNEXT, GetTopWindow, GetWindow, IsWindowVisible,
    };

    let owner_hwnd = owner_hwnd
        .map(|hwnd| hwnd as HWND)
        .unwrap_or(std::ptr::null_mut());
    let mut hwnd = unsafe { GetTopWindow(std::ptr::null_mut()) };

    while !hwnd.is_null() {
        if hwnd != owner_hwnd
            && unsafe { IsWindowVisible(hwnd) } != 0
            && window_class_name(hwnd)
                .is_some_and(|class_name| class_name.starts_with("AE_CApplication_"))
        {
            let process_path = process_image_path(hwnd)?;
            if process_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("AfterFX.exe"))
            {
                return Ok(process_path);
            }
        }

        hwnd = unsafe { GetWindow(hwnd, GW_HWNDNEXT) };
    }

    Err(SendToAfterEffectsError::WindowNotFound)
}

#[cfg(target_os = "windows")]
fn window_class_name(hwnd: windows_sys::Win32::Foundation::HWND) -> Option<String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW;

    let mut buffer = vec![0u16; 256];
    let length = unsafe { GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    (length > 0).then(|| String::from_utf16_lossy(&buffer[..length as usize]))
}

#[cfg(target_os = "windows")]
fn process_image_path(
    hwnd: windows_sys::Win32::Foundation::HWND,
) -> Result<std::path::PathBuf, SendToAfterEffectsError> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::path::PathBuf;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    let mut process_id = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, &mut process_id) };
    if process_id == 0 {
        return Err(SendToAfterEffectsError::ExecutablePathUnavailable);
    }

    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id) };
    if process.is_null() {
        return Err(SendToAfterEffectsError::ExecutablePathUnavailable);
    }

    let mut buffer = vec![0u16; 32768];
    let mut size = buffer.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size) };
    unsafe { CloseHandle(process) };
    if ok == 0 || size == 0 {
        return Err(SendToAfterEffectsError::ExecutablePathUnavailable);
    }

    Ok(PathBuf::from(OsString::from_wide(&buffer[..size as usize])))
}

#[cfg(target_os = "macos")]
fn most_recent_ae_pid_from_window_order() -> Option<i32> {
    use core_foundation::base::TCFType;
    use core_foundation::number::CFNumber;
    use core_foundation::string::CFString;
    use core_graphics::window::{
        create_description_from_array, create_window_list, kCGNullWindowID, kCGWindowLayer,
        kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly, kCGWindowOwnerName,
        kCGWindowOwnerPID,
    };

    let windows = create_window_list(
        kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
        kCGNullWindowID,
    )?;
    let descriptions = create_description_from_array(windows)?;
    let owner_name_key = unsafe { CFString::wrap_under_get_rule(kCGWindowOwnerName) };
    let owner_pid_key = unsafe { CFString::wrap_under_get_rule(kCGWindowOwnerPID) };
    let layer_key = unsafe { CFString::wrap_under_get_rule(kCGWindowLayer) };

    for window in descriptions.iter() {
        let layer_number: CFNumber = window.find(&layer_key)?.downcast()?;
        if layer_number.to_i64()? != 0 {
            continue;
        }

        let owner_name: CFString = window.find(&owner_name_key)?.downcast()?;
        if owner_name.to_string() != "After Effects" {
            continue;
        }

        let owner_pid: CFNumber = window.find(&owner_pid_key)?.downcast()?;
        return Some(owner_pid.to_i64()? as i32);
    }

    None
}

#[cfg(target_os = "macos")]
fn installed_after_effects_app_path() -> Option<std::path::PathBuf> {
    use std::fs;
    use std::path::PathBuf;

    let version_dirs = fs::read_dir("/Applications").ok()?;
    let mut candidates = version_dirs
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with("Adobe After Effects"))
        })
        .flat_map(|dir| {
            fs::read_dir(dir)
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.path())
                .collect::<Vec<_>>()
        })
        .filter(|path| {
            path.extension().and_then(|ext| ext.to_str()) == Some("app")
                && path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| {
                        name.starts_with("Adobe After Effects") && !name.contains("Render Engine")
                    })
        })
        .collect::<Vec<PathBuf>>();
    candidates.sort_by_key(|path| after_effects_sort_key(path));
    candidates.reverse();
    candidates.into_iter().next()
}

#[cfg(target_os = "macos")]
fn after_effects_sort_key(path: &std::path::Path) -> (i32, String) {
    let name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let digits = name
        .chars()
        .rev()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    let version = digits.parse::<i32>().unwrap_or_default();
    (version, name.to_ascii_lowercase())
}
