#![cfg(free_unix)]

use std::{fmt, time::Duration};

use smol_str::SmolStr;

use crate::{
    dpi::Size,
    event::{Event, KeyEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget as RootELW},
    keyboard::{Key, KeyCode},
    platform::{modifier_supplement::KeyEventExtModifierSupplement, scancode::KeyCodeExtScancode},
    window::ActivationToken,
};

pub(crate) use crate::icon::RgbaIcon as PlatformIcon;
pub(crate) use crate::platform_impl::Fullscreen;

pub mod common;
mod eventloop;
mod monitor;
mod window;

pub use eventloop::{EventLoop, EventLoopProxy, EventLoopWindowTarget};
pub use monitor::{MonitorHandle, VideoMode};
pub use window::Window;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Backend {
    #[cfg(x11_platform)]
    X,
    #[cfg(wayland_platform)]
    Wayland,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PlatformSpecificEventLoopAttributes {
    pub(crate) forced_backend: Option<Backend>,
    pub(crate) any_thread: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationName {
    pub general: String,
    pub instance: String,
}

impl ApplicationName {
    pub fn new(general: String, instance: String) -> Self {
        Self { general, instance }
    }
}

#[derive(Clone)]
pub struct PlatformSpecificWindowBuilderAttributes {
    pub name: Option<ApplicationName>,
    pub activation_token: Option<ActivationToken>,
    #[cfg(x11_platform)]
    pub x11: X11WindowBuilderAttributes,
}

#[derive(Clone)]
#[cfg(x11_platform)]
pub struct X11WindowBuilderAttributes {
    pub visual_id: Option<x11rb::protocol::xproto::Visualid>,
    pub screen_id: Option<i32>,
    pub base_size: Option<Size>,
    pub override_redirect: bool,
    // TODO pub x11_window_types: Vec<XWindowType>,
    /// The parent window to embed this window into.
    pub embed_window: Option<x11rb::protocol::xproto::Window>,
}

impl Default for PlatformSpecificWindowBuilderAttributes {
    fn default() -> Self {
        Self {
            name: None,
            activation_token: None,
            #[cfg(x11_platform)]
            x11: X11WindowBuilderAttributes {
                visual_id: None,
                screen_id: None,
                base_size: None,
                override_redirect: false,
                // x11_window_types: vec![XWindowType::Normal],
                embed_window: None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum OsError {
    Misc(&'static str),
}

impl fmt::Display for OsError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match *self {
            OsError::Misc(e) => _f.pad(e),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(u64);

impl From<WindowId> for u64 {
    fn from(window_id: WindowId) -> Self {
        window_id.0
    }
}

impl From<u64> for WindowId {
    fn from(raw_id: u64) -> Self {
        Self(raw_id)
    }
}

impl WindowId {
    pub const unsafe fn dummy() -> Self {
        Self(0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId(usize);

impl DeviceId {
    pub const unsafe fn dummy() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KeyEventExtra {
    pub key_without_modifiers: Key,
    pub text_with_all_modifiers: Option<SmolStr>,
}

impl KeyEventExtModifierSupplement for KeyEvent {
    #[inline]
    fn text_with_all_modifiers(&self) -> Option<&str> {
        self.platform_specific
            .text_with_all_modifiers
            .as_ref()
            .map(|s| s.as_str())
    }

    #[inline]
    fn key_without_modifiers(&self) -> Key {
        self.platform_specific.key_without_modifiers.clone()
    }
}

impl KeyCodeExtScancode for KeyCode {
    fn from_scancode(scancode: u32) -> KeyCode {
        common::keymap::scancode_to_keycode(scancode)
    }

    fn to_scancode(self) -> Option<u32> {
        common::keymap::keycode_to_scancode(self)
    }
}

fn sticky_exit_callback<T, F>(
    evt: Event<T>,
    target: &RootELW<T>,
    control_flow: &mut ControlFlow,
    callback: &mut F,
) where
    F: FnMut(Event<T>, &RootELW<T>, &mut ControlFlow),
{
    // make ControlFlow::ExitWithCode sticky by providing a dummy
    // control flow reference if it is already ExitWithCode.
    if let ControlFlow::ExitWithCode(code) = *control_flow {
        callback(evt, target, &mut ControlFlow::ExitWithCode(code))
    } else {
        callback(evt, target, control_flow)
    }
}

/// Returns the minimum `Option<Duration>`, taking into account that `None`
/// equates to an infinite timeout, not a zero timeout (so can't just use
/// `Option::min`)
fn min_timeout(a: Option<Duration>, b: Option<Duration>) -> Option<Duration> {
    a.map_or(b, |a_timeout| {
        b.map_or(Some(a_timeout), |b_timeout| Some(a_timeout.min(b_timeout)))
    })
}

#[cfg(target_os = "linux")]
fn is_main_thread() -> bool {
    rustix::thread::gettid() == rustix::process::getpid()
}

#[cfg(any(target_os = "dragonfly", target_os = "freebsd", target_os = "openbsd"))]
fn is_main_thread() -> bool {
    use libc::pthread_main_np;

    unsafe { pthread_main_np() == 1 }
}

#[cfg(target_os = "netbsd")]
fn is_main_thread() -> bool {
    std::thread::current().name() == Some("main")
}
