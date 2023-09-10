use crate::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};

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
        Box::new(Vec::new().into_iter())
    }
}

unsafe impl Send for MonitorHandle {}
unsafe impl Sync for MonitorHandle {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoMode;

impl VideoMode {
    #[inline]
    pub fn size(&self) -> PhysicalSize<u32> {
        todo!("VideoMode isn't implemented yet.")
    }

    #[inline]
    pub fn bit_depth(&self) -> u16 {
        todo!("VideoMode isn't implemented yet.")
    }

    #[inline]
    pub fn refresh_rate_millihertz(&self) -> u32 {
        todo!("VideoMode isn't implemented yet.")
    }

    #[inline]
    pub fn monitor(&self) -> MonitorHandle {
        todo!("VideoMode isn't implemented yet.")
    }
}
