//! The red unread count on the taskbar button, as Discord shows it: Windows'
//! overlay icon, drawn here as a red disc with the count in white.

use super::*;

/// Shows `count` mentions and DMs waiting on the taskbar button, or none.
pub(super) fn show(window: &StaticWindow, count: u64) {
    #[cfg(windows)]
    {
        use slint::winit_030::WinitWindowAccessor as _;
        use slint::winit_030::winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let _ = window.window().with_winit_window(|winit| {
            if let Ok(handle) = winit.window_handle()
                && let RawWindowHandle::Win32(handle) = handle.as_raw()
            {
                taskbar_badge::show(handle.hwnd.get(), count);
            }
        });
    }
    #[cfg(not(windows))]
    let _ = (window, count);
}

#[cfg(windows)]
mod taskbar_badge {
    use std::{cell::Cell, ffi::c_void, ptr::null_mut};

    use windows_sys::{
        Win32::{
            Foundation::{HWND, RECT},
            Graphics::Gdi::{
                ANTIALIASED_QUALITY, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CLIP_DEFAULT_PRECIS,
                CreateBitmap, CreateCompatibleDC, CreateDIBSection, CreateFontW, DEFAULT_CHARSET,
                DIB_RGB_COLORS, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteDC, DeleteObject,
                DrawTextW, FW_BOLD, GdiFlush, OUT_DEFAULT_PRECIS, SelectObject, SetBkMode,
                SetTextColor, TRANSPARENT,
            },
            System::Com::{
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
            },
            UI::WindowsAndMessaging::{
                CreateIconIndirect, DestroyIcon, GetSystemMetrics, HICON, ICONINFO, SM_CXSMICON,
            },
        },
        core::GUID,
    };

    const CLSID_TASKBAR_LIST: GUID = GUID::from_u128(0x56fdf344_fd6d_11d0_958a_006097c9a090);
    const IID_TASKBAR_LIST3: GUID = GUID::from_u128(0xea1afb91_9e28_4b86_90e9_9e9f8a5eefaf);
    /// ITaskbarList's HrInit, and ITaskbarList3's SetOverlayIcon, by their
    /// place in the interface's table.
    const HR_INIT: usize = 3;
    const SET_OVERLAY_ICON: usize = 18;

    #[repr(C)]
    struct Object {
        table: *const *const c_void,
    }

    thread_local! {
        static TASKBAR: Cell<*mut Object> = const { Cell::new(null_mut()) };
        /// The count shown, so an unchanged one isn't drawn again.
        static SHOWN: Cell<Option<(isize, u64)>> = const { Cell::new(None) };
    }

    /// Windows' taskbar, made once for this thread.
    fn taskbar() -> Option<*mut Object> {
        TASKBAR.with(|cell| {
            if cell.get().is_null() {
                // SAFETY: COM on this thread, then the taskbar's interface,
                // whose table's entries are called with their own signatures.
                unsafe {
                    let _ = CoInitializeEx(null_mut(), COINIT_APARTMENTTHREADED as _);
                    let mut object: *mut c_void = null_mut();
                    let made = CoCreateInstance(
                        &CLSID_TASKBAR_LIST,
                        null_mut(),
                        CLSCTX_INPROC_SERVER,
                        &IID_TASKBAR_LIST3,
                        &mut object,
                    );
                    if made < 0 || object.is_null() {
                        return None;
                    }
                    let object = object.cast::<Object>();
                    let init: unsafe extern "system" fn(*mut Object) -> i32 =
                        std::mem::transmute(*(*object).table.add(HR_INIT));
                    if init(object) < 0 {
                        return None;
                    }
                    cell.set(object);
                }
            }
            Some(cell.get())
        })
    }

    pub(super) fn show(window: isize, count: u64) {
        if SHOWN.with(Cell::get) == Some((window, count)) {
            return;
        }
        let Some(taskbar) = taskbar() else {
            return;
        };
        // SAFETY: the icon is made and destroyed here; the taskbar keeps its
        // own copy of it.
        unsafe {
            let icon = if count == 0 { null_mut() } else { badge(count) };
            let label: Vec<u16> = format!("{count} unread")
                .encode_utf16()
                .chain([0])
                .collect();
            let set: unsafe extern "system" fn(*mut Object, HWND, HICON, *const u16) -> i32 =
                std::mem::transmute(*(*taskbar).table.add(SET_OVERLAY_ICON));
            let done = set(
                taskbar,
                window as HWND,
                icon,
                if icon.is_null() { std::ptr::null() } else { label.as_ptr() },
            );
            if !icon.is_null() {
                DestroyIcon(icon);
            }
            if done >= 0 {
                SHOWN.with(|shown| shown.set(Some((window, count))));
            }
        }
    }

    /// A red disc with the count in white, at the size of a small icon.
    unsafe fn badge(count: u64) -> HICON {
        // SAFETY: each object made here is deleted before returning.
        unsafe {
            let size = GetSystemMetrics(SM_CXSMICON).max(16);
            let mut info: BITMAPINFO = std::mem::zeroed();
            info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
            info.bmiHeader.biWidth = size;
            info.bmiHeader.biHeight = -size;
            info.bmiHeader.biPlanes = 1;
            info.bmiHeader.biBitCount = 32;
            info.bmiHeader.biCompression = BI_RGB as _;
            let dc = CreateCompatibleDC(null_mut());
            let mut bits: *mut c_void = null_mut();
            let colour = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
            if colour.is_null() || bits.is_null() {
                DeleteDC(dc);
                return null_mut();
            }
            let before = SelectObject(dc, colour);
            // The count, white on black, says how much white each pixel takes.
            let face: Vec<u16> = "Segoe UI".encode_utf16().chain([0]).collect();
            let font = CreateFontW(
                -(size * 11 / 16),
                0,
                0,
                0,
                FW_BOLD as _,
                0,
                0,
                0,
                DEFAULT_CHARSET as _,
                OUT_DEFAULT_PRECIS as _,
                CLIP_DEFAULT_PRECIS as _,
                ANTIALIASED_QUALITY as _,
                0,
                face.as_ptr(),
            );
            let before_font = SelectObject(dc, font);
            SetBkMode(dc, TRANSPARENT as _);
            SetTextColor(dc, 0x00ff_ffff);
            let mut text: Vec<u16> = if count > 9 { "9+".to_owned() } else { count.to_string() }
                .encode_utf16()
                .collect();
            let mut area = RECT {
                left: 0,
                top: 0,
                right: size,
                bottom: size,
            };
            DrawTextW(
                dc,
                text.as_mut_ptr(),
                text.len() as i32,
                &mut area,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            GdiFlush();
            let pixels = std::slice::from_raw_parts_mut(bits.cast::<u8>(), (size * size * 4) as usize);
            let centre = size as f32 / 2.0;
            for y in 0..size {
                for x in 0..size {
                    let at = ((y * size + x) * 4) as usize;
                    let white = f32::from(pixels[at].max(pixels[at + 1]).max(pixels[at + 2])) / 255.0;
                    let dx = x as f32 + 0.5 - centre;
                    let dy = y as f32 + 0.5 - centre;
                    let inside = (centre - (dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0);
                    // Discord's red, #f23f43, toward white where the count is.
                    let mix = |red: f32| (red + (255.0 - red) * white).round() as u8;
                    pixels[at] = mix(67.0);
                    pixels[at + 1] = mix(63.0);
                    pixels[at + 2] = mix(242.0);
                    pixels[at + 3] = (inside * 255.0).round() as u8;
                }
            }
            SelectObject(dc, before_font);
            DeleteObject(font);
            SelectObject(dc, before);
            let mask = CreateBitmap(size, size, 1, 1, std::ptr::null());
            let icon_info = ICONINFO {
                fIcon: 1,
                xHotspot: 0,
                yHotspot: 0,
                hbmMask: mask,
                hbmColor: colour,
            };
            let icon = CreateIconIndirect(&icon_info);
            DeleteObject(mask);
            DeleteObject(colour);
            DeleteDC(dc);
            icon
        }
    }
}
