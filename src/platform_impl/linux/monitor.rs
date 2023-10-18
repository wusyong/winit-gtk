use crate::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use gdk::prelude::MonitorExt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MonitorHandle {
    pub(crate) monitor: gdk::Monitor,
}

impl MonitorHandle {
    pub fn new(display: &gdk::Display, number: i32) -> Self {
        let monitor = display.monitor(number).unwrap();
        Self { monitor }
    }

    #[inline]
    pub fn name(&self) -> Option<String> {
        self.monitor.model().map(|s| s.as_str().to_string())
    }

    #[inline]
    pub fn size(&self) -> PhysicalSize<u32> {
        let rect = self.monitor.geometry();
        LogicalSize {
            width: rect.width() as u32,
            height: rect.height() as u32,
        }
        .to_physical(self.scale_factor())
    }

    #[inline]
    pub fn position(&self) -> PhysicalPosition<i32> {
        let rect = self.monitor.geometry();
        LogicalPosition {
            x: rect.x(),
            y: rect.y(),
        }
        .to_physical(self.scale_factor())
    }

    #[inline]
    pub fn refresh_rate_millihertz(&self) -> Option<u32> {
        Some(self.monitor.refresh_rate() as u32)
    }

    #[inline]
    pub fn scale_factor(&self) -> f64 {
        self.monitor.scale_factor() as f64
    }

    #[inline]
    pub fn video_modes(&self) -> Box<dyn Iterator<Item = VideoMode>> {
        Box::new(
            vec![VideoMode {
                monitor: self.monitor.clone(),
            }]
            .into_iter(),
        )
    }
}

unsafe impl Send for MonitorHandle {}
unsafe impl Sync for MonitorHandle {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoMode {
    /// gdk::Screen is deprecated. We make VideoMode and MonitorHandle
    /// being the same type. If we want to enrich this feature. We will
    /// need to look for x11/wayland implementations.
    pub(crate) monitor: gdk::Monitor,
}

impl VideoMode {
    #[inline]
    pub fn size(&self) -> PhysicalSize<u32> {
        let rect = self.monitor.geometry();
        LogicalSize {
            width: rect.width() as u32,
            height: rect.height() as u32,
        }
        .to_physical(self.monitor.scale_factor() as f64)
    }

    #[inline]
    pub fn bit_depth(&self) -> u16 {
        32
    }

    #[inline]
    pub fn refresh_rate_millihertz(&self) -> u32 {
        self.monitor.refresh_rate() as u32
    }

    #[inline]
    pub fn monitor(&self) -> MonitorHandle {
        MonitorHandle {
            monitor: self.monitor.clone(),
        }
    }
}

unsafe impl Send for VideoMode {}
unsafe impl Sync for VideoMode {}
