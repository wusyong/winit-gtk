#![cfg(free_unix)]

use std::fmt;

use crate::event::DeviceId as RootDeviceId;

pub(crate) use crate::icon::RgbaIcon as PlatformIcon;
pub(self) use crate::platform_impl::Fullscreen;

mod eventloop;
mod keyboard;
mod monitor;
mod util;
mod window;

pub use eventloop::{EventLoop, EventLoopProxy, EventLoopWindowTarget};
use gdk_pixbuf::{Colorspace, Pixbuf};
pub use monitor::{MonitorHandle, VideoMode};
pub use window::Window;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Backend {
    // #[cfg(x11_platform)]
    // X,
    // #[cfg(wayland_platform)]
    // Wayland,
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
    pub parent: Option<gtk::Window>,
    pub skip_taskbar: bool,
    pub auto_transparent: bool,
    pub double_buffered: bool,
    pub app_paintable: bool,
    pub rgba_visual: bool,
    pub default_vbox: bool,
}

impl Default for PlatformSpecificWindowBuilderAttributes {
    fn default() -> Self {
        Self {
            name: None,
            parent: None,
            skip_taskbar: Default::default(),
            auto_transparent: true,
            double_buffered: true,
            app_paintable: false,
            rgba_visual: false,
            default_vbox: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum OsError {
    // Misc(&'static str),
}

impl fmt::Display for OsError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        // match *self {
        //     OsError::Misc(e) => _f.pad(e),
        // }
        Ok(())
    }
}

impl From<PlatformIcon> for Pixbuf {
    fn from(icon: PlatformIcon) -> Self {
        let height = icon.height as i32;
        let width = icon.width as i32;
        let row_stride = Pixbuf::calculate_rowstride(Colorspace::Rgb, true, 8, width, height);
        Pixbuf::from_mut_slice(
            icon.rgba,
            gdk_pixbuf::Colorspace::Rgb,
            true,
            8,
            width,
            height,
            row_stride,
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub u64);

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

// TODO: currently we use a dummy device id, find if we can get device id from gtk
pub(crate) const DEVICE_ID: RootDeviceId = RootDeviceId(DeviceId(0));
