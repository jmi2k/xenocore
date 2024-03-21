use core::{ffi::CStr, mem, ptr};

use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Gdi::{GetDC, ReleaseDC, HDC},
    System::LibraryLoader::GetModuleHandleA,
    UI::WindowsAndMessaging::{
        CreateWindowExA, DefWindowProcA, DestroyWindow, DispatchMessageA, GetClientRect,
        GetWindowLongPtrA, LoadCursorA, PeekMessageA, PostQuitMessage, RegisterClassA,
        SetWindowLongPtrA, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW, MSG,
        PM_REMOVE, WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_QUIT, WNDCLASSA,
        WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    },
};

use crate::Event;

#[cfg(feature = "gl")]
use windows_sys::Win32::Graphics::OpenGL::{
    wglCreateContext, wglDeleteContext, wglMakeCurrent, ChoosePixelFormat, SetPixelFormat, HGLRC,
    PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_SUPPORT_OPENGL, PFD_TYPE_RGBA, PIXELFORMATDESCRIPTOR,
};

pub struct Window {
    pub hwnd: HWND,
    pub hdc: HDC,

    #[cfg(feature = "gl")]
    pub hglrc: HGLRC,
}

impl Window {
    pub fn new(name: &CStr) -> Self {
        let instance = unsafe { GetModuleHandleA(ptr::null()) };
        assert!(instance != 0);

        let class = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(message_callback),
            hInstance: instance,
            hCursor: unsafe { LoadCursorA(0, IDC_ARROW as _) },
            lpszClassName: c"window".as_ptr() as _,
            ..unsafe { mem::zeroed() }
        };

        let atom = unsafe { RegisterClassA(&class) };
        assert!(atom != 0);

        let hwnd = unsafe {
            CreateWindowExA(
                0,
                class.lpszClassName,
                name.as_ptr() as _,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                0,
                0,
                instance,
                ptr::null(),
            )
        };

        assert!(hwnd != 0);

        let hdc = unsafe { GetDC(hwnd) };
        assert!(hdc != 0);

        #[cfg(feature = "gl")]
        {
            let desc = PIXELFORMATDESCRIPTOR {
                nSize: mem::size_of::<PIXELFORMATDESCRIPTOR>() as _,
                nVersion: 1,
                dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
                iPixelType: PFD_TYPE_RGBA,
                cColorBits: 24,
                cDepthBits: 32,
                ..unsafe { mem::zeroed() }
            };

            let idx = unsafe { ChoosePixelFormat(hdc, &desc) };
            assert!(idx != 0);

            let success = unsafe { SetPixelFormat(hdc, idx, &desc) };
            assert!(success != 0);
        }

        #[cfg(feature = "gl")]
        let hglrc = unsafe { wglCreateContext(hdc) };

        #[cfg(feature = "gl")]
        assert!(hglrc != 0);

        unsafe { wglMakeCurrent(hdc, hglrc) };

        Self {
            hwnd,
            hdc,

            #[cfg(feature = "gl")]
            hglrc,
        }
    }

    pub fn inner_size(&self) -> [u32; 2] {
        let mut rect = unsafe { mem::zeroed() };

        let success = unsafe { GetClientRect(self.hwnd, &mut rect) };
        assert!(success != 0);

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        [width as _, height as _]
    }

    pub fn event_loop(&self, mut cb: impl FnMut(Event)) {
        let mut message = unsafe { mem::zeroed::<MSG>() };
        let fat_pointer = &mut cb as *mut dyn FnMut(Event);
        let user_pointer = &fat_pointer as *const _;

        unsafe { SetWindowLongPtrA(self.hwnd, GWLP_USERDATA, user_pointer as _) };

        while message.message != WM_QUIT {
            if unsafe { PeekMessageA(&mut message, 0, 0, 0, PM_REMOVE) } == 0 {
                cb(Event::Idle);
                continue;
            }

            unsafe { DispatchMessageA(&message) };
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        #[cfg(feature = "gl")]
        unsafe {
            wglMakeCurrent(0, 0);
            wglDeleteContext(self.hglrc);
        }

        unsafe { ReleaseDC(self.hwnd, self.hdc); };
    }
}

extern "system" fn message_callback(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    type UserPtr = *const *mut dyn FnMut(Event);
    let user_pointer = unsafe { GetWindowLongPtrA(hwnd, GWLP_USERDATA) } as UserPtr;
    let callback = unsafe { user_pointer.as_ref() };

    match (callback, message) {
        (Some(cb), WM_KEYDOWN) => {
            unsafe { (**cb)(Event::Press(w as _)) };
            0
        }

        (Some(cb), WM_KEYUP) => {
            unsafe { (**cb)(Event::Release(w as _)) };
            0
        }

        (_, WM_CLOSE) => {
            unsafe { DestroyWindow(hwnd) };
            0
        }

        (_, WM_DESTROY) => {
            unsafe { PostQuitMessage(0) };
            0
        }

        _ => unsafe { DefWindowProcA(hwnd, message, w, l) },
    }
}

#[cfg(feature = "gl")]
#[macro_export]
macro_rules! wgl_load {
    ($name:literal, $($ty:tt)*) => {{
        use ::core::{ffi::CStr, mem};
        use ::windows_sys::Win32::Graphics::OpenGL::wglGetProcAddress;

        let name: &CStr = $name;
        let result = wglGetProcAddress(name.as_ptr() as _);

        #[allow(clippy::useless_transmute)]
        result.map(|proc| mem::transmute::<_, unsafe extern "system" $($ty)*>(proc))
    }};
}

pub mod key {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse as km;

    pub const LBUTTON: usize = km::VK_LBUTTON as _;
    pub const RBUTTON: usize = km::VK_RBUTTON as _;
    pub const CANCEL: usize = km::VK_CANCEL as _;
    pub const MBUTTON: usize = km::VK_MBUTTON as _;
    pub const XBUTTON1: usize = km::VK_XBUTTON1 as _;
    pub const XBUTTON2: usize = km::VK_XBUTTON2 as _;
    pub const BACK: usize = km::VK_BACK as _;
    pub const TAB: usize = km::VK_TAB as _;
    pub const CLEAR: usize = km::VK_CLEAR as _;
    pub const RETURN: usize = km::VK_RETURN as _;
    pub const SHIFT: usize = km::VK_SHIFT as _;
    pub const CONTROL: usize = km::VK_CONTROL as _;
    pub const MENU: usize = km::VK_MENU as _;
    pub const PAUSE: usize = km::VK_PAUSE as _;
    pub const CAPITAL: usize = km::VK_CAPITAL as _;
    pub const HANGEUL: usize = km::VK_HANGEUL as _;
    pub const HANGUL: usize = km::VK_HANGUL as _;
    pub const KANA: usize = km::VK_KANA as _;
    pub const IME_ON: usize = km::VK_IME_ON as _;
    pub const JUNJA: usize = km::VK_JUNJA as _;
    pub const FINAL: usize = km::VK_FINAL as _;
    pub const HANJA: usize = km::VK_HANJA as _;
    pub const KANJI: usize = km::VK_KANJI as _;
    pub const IME_OFF: usize = km::VK_IME_OFF as _;
    pub const ESCAPE: usize = km::VK_ESCAPE as _;
    pub const CONVERT: usize = km::VK_CONVERT as _;
    pub const NONCONVERT: usize = km::VK_NONCONVERT as _;
    pub const ACCEPT: usize = km::VK_ACCEPT as _;
    pub const MODECHANGE: usize = km::VK_MODECHANGE as _;
    pub const SPACE: usize = km::VK_SPACE as _;
    pub const PRIOR: usize = km::VK_PRIOR as _;
    pub const NEXT: usize = km::VK_NEXT as _;
    pub const END: usize = km::VK_END as _;
    pub const HOME: usize = km::VK_HOME as _;
    pub const LEFT: usize = km::VK_LEFT as _;
    pub const UP: usize = km::VK_UP as _;
    pub const RIGHT: usize = km::VK_RIGHT as _;
    pub const DOWN: usize = km::VK_DOWN as _;
    pub const SELECT: usize = km::VK_SELECT as _;
    pub const PRINT: usize = km::VK_PRINT as _;
    pub const EXECUTE: usize = km::VK_EXECUTE as _;
    pub const SNAPSHOT: usize = km::VK_SNAPSHOT as _;
    pub const INSERT: usize = km::VK_INSERT as _;
    pub const DELETE: usize = km::VK_DELETE as _;
    pub const HELP: usize = km::VK_HELP as _;
    pub const NUM0: usize = km::VK_0 as _;
    pub const NUM1: usize = km::VK_1 as _;
    pub const NUM2: usize = km::VK_2 as _;
    pub const NUM3: usize = km::VK_3 as _;
    pub const NUM4: usize = km::VK_4 as _;
    pub const NUM5: usize = km::VK_5 as _;
    pub const NUM6: usize = km::VK_6 as _;
    pub const NUM7: usize = km::VK_7 as _;
    pub const NUM8: usize = km::VK_8 as _;
    pub const NUM9: usize = km::VK_9 as _;
    pub const A: usize = km::VK_A as _;
    pub const B: usize = km::VK_B as _;
    pub const C: usize = km::VK_C as _;
    pub const D: usize = km::VK_D as _;
    pub const E: usize = km::VK_E as _;
    pub const F: usize = km::VK_F as _;
    pub const G: usize = km::VK_G as _;
    pub const H: usize = km::VK_H as _;
    pub const I: usize = km::VK_I as _;
    pub const J: usize = km::VK_J as _;
    pub const K: usize = km::VK_K as _;
    pub const L: usize = km::VK_L as _;
    pub const M: usize = km::VK_M as _;
    pub const N: usize = km::VK_N as _;
    pub const O: usize = km::VK_O as _;
    pub const P: usize = km::VK_P as _;
    pub const Q: usize = km::VK_Q as _;
    pub const R: usize = km::VK_R as _;
    pub const S: usize = km::VK_S as _;
    pub const T: usize = km::VK_T as _;
    pub const U: usize = km::VK_U as _;
    pub const V: usize = km::VK_V as _;
    pub const W: usize = km::VK_W as _;
    pub const X: usize = km::VK_X as _;
    pub const Y: usize = km::VK_Y as _;
    pub const Z: usize = km::VK_Z as _;
    pub const LWIN: usize = km::VK_LWIN as _;
    pub const RWIN: usize = km::VK_RWIN as _;
    pub const APPS: usize = km::VK_APPS as _;
    pub const SLEEP: usize = km::VK_SLEEP as _;
    pub const NUMPAD0: usize = km::VK_NUMPAD0 as _;
    pub const NUMPAD1: usize = km::VK_NUMPAD1 as _;
    pub const NUMPAD2: usize = km::VK_NUMPAD2 as _;
    pub const NUMPAD3: usize = km::VK_NUMPAD3 as _;
    pub const NUMPAD4: usize = km::VK_NUMPAD4 as _;
    pub const NUMPAD5: usize = km::VK_NUMPAD5 as _;
    pub const NUMPAD6: usize = km::VK_NUMPAD6 as _;
    pub const NUMPAD7: usize = km::VK_NUMPAD7 as _;
    pub const NUMPAD8: usize = km::VK_NUMPAD8 as _;
    pub const NUMPAD9: usize = km::VK_NUMPAD9 as _;
    pub const MULTIPLY: usize = km::VK_MULTIPLY as _;
    pub const ADD: usize = km::VK_ADD as _;
    pub const SEPARATOR: usize = km::VK_SEPARATOR as _;
    pub const SUBTRACT: usize = km::VK_SUBTRACT as _;
    pub const DECIMAL: usize = km::VK_DECIMAL as _;
    pub const DIVIDE: usize = km::VK_DIVIDE as _;
    pub const F1: usize = km::VK_F1 as _;
    pub const F2: usize = km::VK_F2 as _;
    pub const F3: usize = km::VK_F3 as _;
    pub const F4: usize = km::VK_F4 as _;
    pub const F5: usize = km::VK_F5 as _;
    pub const F6: usize = km::VK_F6 as _;
    pub const F7: usize = km::VK_F7 as _;
    pub const F8: usize = km::VK_F8 as _;
    pub const F9: usize = km::VK_F9 as _;
    pub const F10: usize = km::VK_F10 as _;
    pub const F11: usize = km::VK_F11 as _;
    pub const F12: usize = km::VK_F12 as _;
    pub const F13: usize = km::VK_F13 as _;
    pub const F14: usize = km::VK_F14 as _;
    pub const F15: usize = km::VK_F15 as _;
    pub const F16: usize = km::VK_F16 as _;
    pub const F17: usize = km::VK_F17 as _;
    pub const F18: usize = km::VK_F18 as _;
    pub const F19: usize = km::VK_F19 as _;
    pub const F20: usize = km::VK_F20 as _;
    pub const F21: usize = km::VK_F21 as _;
    pub const F22: usize = km::VK_F22 as _;
    pub const F23: usize = km::VK_F23 as _;
    pub const F24: usize = km::VK_F24 as _;
    pub const NAVIGATION_VIEW: usize = km::VK_NAVIGATION_VIEW as _;
    pub const NAVIGATION_MENU: usize = km::VK_NAVIGATION_MENU as _;
    pub const NAVIGATION_UP: usize = km::VK_NAVIGATION_UP as _;
    pub const NAVIGATION_DOWN: usize = km::VK_NAVIGATION_DOWN as _;
    pub const NAVIGATION_LEFT: usize = km::VK_NAVIGATION_LEFT as _;
    pub const NAVIGATION_RIGHT: usize = km::VK_NAVIGATION_RIGHT as _;
    pub const NAVIGATION_ACCEPT: usize = km::VK_NAVIGATION_ACCEPT as _;
    pub const NAVIGATION_CANCEL: usize = km::VK_NAVIGATION_CANCEL as _;
    pub const NUMLOCK: usize = km::VK_NUMLOCK as _;
    pub const SCROLL: usize = km::VK_SCROLL as _;
    pub const OEM_FJ_JISHO: usize = km::VK_OEM_FJ_JISHO as _;
    pub const OEM_NEC_EQUAL: usize = km::VK_OEM_NEC_EQUAL as _;
    pub const OEM_FJ_MASSHOU: usize = km::VK_OEM_FJ_MASSHOU as _;
    pub const OEM_FJ_TOUROKU: usize = km::VK_OEM_FJ_TOUROKU as _;
    pub const OEM_FJ_LOYA: usize = km::VK_OEM_FJ_LOYA as _;
    pub const OEM_FJ_ROYA: usize = km::VK_OEM_FJ_ROYA as _;
    pub const LSHIFT: usize = km::VK_LSHIFT as _;
    pub const RSHIFT: usize = km::VK_RSHIFT as _;
    pub const LCONTROL: usize = km::VK_LCONTROL as _;
    pub const RCONTROL: usize = km::VK_RCONTROL as _;
    pub const LMENU: usize = km::VK_LMENU as _;
    pub const RMENU: usize = km::VK_RMENU as _;
    pub const BROWSER_BACK: usize = km::VK_BROWSER_BACK as _;
    pub const BROWSER_FORWARD: usize = km::VK_BROWSER_FORWARD as _;
    pub const BROWSER_REFRESH: usize = km::VK_BROWSER_REFRESH as _;
    pub const BROWSER_STOP: usize = km::VK_BROWSER_STOP as _;
    pub const BROWSER_SEARCH: usize = km::VK_BROWSER_SEARCH as _;
    pub const BROWSER_FAVORITES: usize = km::VK_BROWSER_FAVORITES as _;
    pub const BROWSER_HOME: usize = km::VK_BROWSER_HOME as _;
    pub const VOLUME_MUTE: usize = km::VK_VOLUME_MUTE as _;
    pub const VOLUME_DOWN: usize = km::VK_VOLUME_DOWN as _;
    pub const VOLUME_UP: usize = km::VK_VOLUME_UP as _;
    pub const MEDIA_NEXT_TRACK: usize = km::VK_MEDIA_NEXT_TRACK as _;
    pub const MEDIA_PREV_TRACK: usize = km::VK_MEDIA_PREV_TRACK as _;
    pub const MEDIA_STOP: usize = km::VK_MEDIA_STOP as _;
    pub const MEDIA_PLAY_PAUSE: usize = km::VK_MEDIA_PLAY_PAUSE as _;
    pub const LAUNCH_MAIL: usize = km::VK_LAUNCH_MAIL as _;
    pub const LAUNCH_MEDIA_SELECT: usize = km::VK_LAUNCH_MEDIA_SELECT as _;
    pub const LAUNCH_APP1: usize = km::VK_LAUNCH_APP1 as _;
    pub const LAUNCH_APP2: usize = km::VK_LAUNCH_APP2 as _;
    pub const OEM_1: usize = km::VK_OEM_1 as _;
    pub const OEM_PLUS: usize = km::VK_OEM_PLUS as _;
    pub const OEM_COMMA: usize = km::VK_OEM_COMMA as _;
    pub const OEM_MINUS: usize = km::VK_OEM_MINUS as _;
    pub const OEM_PERIOD: usize = km::VK_OEM_PERIOD as _;
    pub const OEM_2: usize = km::VK_OEM_2 as _;
    pub const OEM_3: usize = km::VK_OEM_3 as _;
    pub const ABNT_C1: usize = km::VK_ABNT_C1 as _;
    pub const ABNT_C2: usize = km::VK_ABNT_C2 as _;
    pub const GAMEPAD_A: usize = km::VK_GAMEPAD_A as _;
    pub const GAMEPAD_B: usize = km::VK_GAMEPAD_B as _;
    pub const GAMEPAD_X: usize = km::VK_GAMEPAD_X as _;
    pub const GAMEPAD_Y: usize = km::VK_GAMEPAD_Y as _;
    pub const GAMEPAD_RIGHT_SHOULDER: usize = km::VK_GAMEPAD_RIGHT_SHOULDER as _;
    pub const GAMEPAD_LEFT_SHOULDER: usize = km::VK_GAMEPAD_LEFT_SHOULDER as _;
    pub const GAMEPAD_LEFT_TRIGGER: usize = km::VK_GAMEPAD_LEFT_TRIGGER as _;
    pub const GAMEPAD_RIGHT_TRIGGER: usize = km::VK_GAMEPAD_RIGHT_TRIGGER as _;
    pub const GAMEPAD_DPAD_UP: usize = km::VK_GAMEPAD_DPAD_UP as _;
    pub const GAMEPAD_DPAD_DOWN: usize = km::VK_GAMEPAD_DPAD_DOWN as _;
    pub const GAMEPAD_DPAD_LEFT: usize = km::VK_GAMEPAD_DPAD_LEFT as _;
    pub const GAMEPAD_DPAD_RIGHT: usize = km::VK_GAMEPAD_DPAD_RIGHT as _;
    pub const GAMEPAD_MENU: usize = km::VK_GAMEPAD_MENU as _;
    pub const GAMEPAD_VIEW: usize = km::VK_GAMEPAD_VIEW as _;
    pub const GAMEPAD_LEFT_THUMBSTICK_BUTTON: usize = km::VK_GAMEPAD_LEFT_THUMBSTICK_BUTTON as _;
    pub const GAMEPAD_RIGHT_THUMBSTICK_BUTTON: usize = km::VK_GAMEPAD_RIGHT_THUMBSTICK_BUTTON as _;
    pub const GAMEPAD_LEFT_THUMBSTICK_UP: usize = km::VK_GAMEPAD_LEFT_THUMBSTICK_UP as _;
    pub const GAMEPAD_LEFT_THUMBSTICK_DOWN: usize = km::VK_GAMEPAD_LEFT_THUMBSTICK_DOWN as _;
    pub const GAMEPAD_LEFT_THUMBSTICK_RIGHT: usize = km::VK_GAMEPAD_LEFT_THUMBSTICK_RIGHT as _;
    pub const GAMEPAD_LEFT_THUMBSTICK_LEFT: usize = km::VK_GAMEPAD_LEFT_THUMBSTICK_LEFT as _;
    pub const GAMEPAD_RIGHT_THUMBSTICK_UP: usize = km::VK_GAMEPAD_RIGHT_THUMBSTICK_UP as _;
    pub const GAMEPAD_RIGHT_THUMBSTICK_DOWN: usize = km::VK_GAMEPAD_RIGHT_THUMBSTICK_DOWN as _;
    pub const GAMEPAD_RIGHT_THUMBSTICK_RIGHT: usize = km::VK_GAMEPAD_RIGHT_THUMBSTICK_RIGHT as _;
    pub const GAMEPAD_RIGHT_THUMBSTICK_LEFT: usize = km::VK_GAMEPAD_RIGHT_THUMBSTICK_LEFT as _;
    pub const OEM_4: usize = km::VK_OEM_4 as _;
    pub const OEM_5: usize = km::VK_OEM_5 as _;
    pub const OEM_6: usize = km::VK_OEM_6 as _;
    pub const OEM_7: usize = km::VK_OEM_7 as _;
    pub const OEM_8: usize = km::VK_OEM_8 as _;
    pub const OEM_AX: usize = km::VK_OEM_AX as _;
    pub const OEM_102: usize = km::VK_OEM_102 as _;
    pub const ICO_HELP: usize = km::VK_ICO_HELP as _;
    pub const ICO_00: usize = km::VK_ICO_00 as _;
    pub const PROCESSKEY: usize = km::VK_PROCESSKEY as _;
    pub const ICO_CLEAR: usize = km::VK_ICO_CLEAR as _;
    pub const PACKET: usize = km::VK_PACKET as _;
    pub const OEM_RESET: usize = km::VK_OEM_RESET as _;
    pub const OEM_JUMP: usize = km::VK_OEM_JUMP as _;
    pub const OEM_PA1: usize = km::VK_OEM_PA1 as _;
    pub const OEM_PA2: usize = km::VK_OEM_PA2 as _;
    pub const OEM_PA3: usize = km::VK_OEM_PA3 as _;
    pub const OEM_WSCTRL: usize = km::VK_OEM_WSCTRL as _;
    pub const OEM_CUSEL: usize = km::VK_OEM_CUSEL as _;
    pub const DBE_ALPHANUMERIC: usize = km::VK_DBE_ALPHANUMERIC as _;
    pub const OEM_ATTN: usize = km::VK_OEM_ATTN as _;
    pub const DBE_KATAKANA: usize = km::VK_DBE_KATAKANA as _;
    pub const OEM_FINISH: usize = km::VK_OEM_FINISH as _;
    pub const DBE_HIRAGANA: usize = km::VK_DBE_HIRAGANA as _;
    pub const OEM_COPY: usize = km::VK_OEM_COPY as _;
    pub const DBE_SBCSCHAR: usize = km::VK_DBE_SBCSCHAR as _;
    pub const OEM_AUTO: usize = km::VK_OEM_AUTO as _;
    pub const DBE_DBCSCHAR: usize = km::VK_DBE_DBCSCHAR as _;
    pub const OEM_ENLW: usize = km::VK_OEM_ENLW as _;
    pub const DBE_ROMAN: usize = km::VK_DBE_ROMAN as _;
    pub const OEM_BACKTAB: usize = km::VK_OEM_BACKTAB as _;
    pub const ATTN: usize = km::VK_ATTN as _;
    pub const DBE_NOROMAN: usize = km::VK_DBE_NOROMAN as _;
    pub const CRSEL: usize = km::VK_CRSEL as _;
    pub const DBE_ENTERWORDREGISTERMODE: usize = km::VK_DBE_ENTERWORDREGISTERMODE as _;
    pub const DBE_ENTERIMECONFIGMODE: usize = km::VK_DBE_ENTERIMECONFIGMODE as _;
    pub const EXSEL: usize = km::VK_EXSEL as _;
    pub const DBE_FLUSHSTRING: usize = km::VK_DBE_FLUSHSTRING as _;
    pub const EREOF: usize = km::VK_EREOF as _;
    pub const DBE_CODEINPUT: usize = km::VK_DBE_CODEINPUT as _;
    pub const PLAY: usize = km::VK_PLAY as _;
    pub const DBE_NOCODEINPUT: usize = km::VK_DBE_NOCODEINPUT as _;
    pub const ZOOM: usize = km::VK_ZOOM as _;
    pub const DBE_DETERMINESTRING: usize = km::VK_DBE_DETERMINESTRING as _;
    pub const NONAME: usize = km::VK_NONAME as _;
    pub const DBE_ENTERDLGCONVERSIONMODE: usize = km::VK_DBE_ENTERDLGCONVERSIONMODE as _;
    pub const PA1: usize = km::VK_PA1 as _;
    pub const OEM_CLEAR: usize = km::VK_OEM_CLEAR as _;
}
