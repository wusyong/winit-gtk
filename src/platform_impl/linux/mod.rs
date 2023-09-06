#![cfg(free_unix)]

use std::fmt;

use crate::{
    event::Event,
    event_loop::{ControlFlow, EventLoopWindowTarget as RootELW},
};

pub(crate) use crate::icon::RgbaIcon as PlatformIcon;
pub(self) use crate::platform_impl::Fullscreen;

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
}

impl Default for PlatformSpecificWindowBuilderAttributes {
    fn default() -> Self {
        Self { name: None }
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

fn sticky_exit_callback<T, F>(
    evt: Event<'_, T>,
    target: &RootELW<T>,
    control_flow: &mut ControlFlow,
    callback: &mut F,
) where
    F: FnMut(Event<'_, T>, &RootELW<T>, &mut ControlFlow),
{
    // make ControlFlow::ExitWithCode sticky by providing a dummy
    // control flow reference if it is already ExitWithCode.
    if let ControlFlow::ExitWithCode(code) = *control_flow {
        callback(evt, target, &mut ControlFlow::ExitWithCode(code))
    } else {
        callback(evt, target, control_flow)
    }
}

#[cfg(target_os = "linux")]
fn is_main_thread() -> bool {
    use libc::{c_long, getpid, syscall, SYS_gettid};

    unsafe { syscall(SYS_gettid) == getpid() as c_long }
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
