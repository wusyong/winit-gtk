use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    sync::atomic::{AtomicBool, AtomicI32, Ordering},
};

use gtk::traits::WidgetExt;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size},
    error::{ExternalError, NotSupportedError, OsError as RootOsError},
    platform_impl::WindowId,
    window::{
        CursorGrabMode, CursorIcon, Icon, ImePurpose, ResizeDirection, Theme, UserAttentionType,
        WindowAttributes, WindowButtons, WindowLevel,
    },
};

use super::{
    EventLoopWindowTarget, Fullscreen, MonitorHandle, PlatformSpecificWindowBuilderAttributes,
};

pub(crate) enum WindowRequest {
    Title(String),
    Position((i32, i32)),
    Size((i32, i32)),
    // SizeConstraints(WindowSizeConstraints),
    Visible(bool),
    Focus,
    Resizable(bool),
    Closable(bool),
    Minimized(bool),
    Maximized(bool),
    DragWindow,
    Fullscreen(Option<Fullscreen>),
    Decorations(bool),
    AlwaysOnBottom(bool),
    AlwaysOnTop(bool),
    // WindowIcon(Option<Icon>),
    // UserAttention(Option<UserAttentionType>),
    SetSkipTaskbar(bool),
    // CursorIcon(Option<CursorIcon>),
    CursorPosition((i32, i32)),
    CursorIgnoreEvents(bool),
    WireUpEvents {
        transparent: bool,
        cursor_moved: bool,
    },
    SetVisibleOnAllWorkspaces(bool),
    // ProgressBarState(ProgressBarState),
}

pub struct Window {
    /// Window id.
    pub(crate) window_id: WindowId,
    /// Gtk application window.
    pub(crate) window: gtk::ApplicationWindow,
    pub(crate) default_vbox: Option<gtk::Box>,
    /// Window requests sender
    pub(crate) window_requests_tx: glib::Sender<(WindowId, WindowRequest)>,
    scale_factor: Rc<AtomicI32>,
    position: Rc<(AtomicI32, AtomicI32)>,
    size: Rc<(AtomicI32, AtomicI32)>,
    maximized: Rc<AtomicBool>,
    minimized: Rc<AtomicBool>,
    fullscreen: RefCell<Option<Fullscreen>>,
    // inner_size_constraints: RefCell<WindowSizeConstraints>,
    /// Draw event Sender
    draw_tx: crossbeam_channel::Sender<WindowId>,
}
impl Window {
    #[inline]
    pub(crate) fn new<T>(
        window_target: &EventLoopWindowTarget<T>,
        attribs: WindowAttributes,
        pl_attribs: PlatformSpecificWindowBuilderAttributes,
    ) -> Result<Self, RootOsError> {
        todo!()
    }

    pub(crate) fn maybe_queue_on_main(&self, f: impl FnOnce(&Self) + Send + 'static) {
        f(self)
    }

    pub(crate) fn maybe_wait_on_main<R: Send>(&self, f: impl FnOnce(&Self) -> R + Send) -> R {
        f(self)
    }

    #[inline]
    pub fn id(&self) -> WindowId {
        self.window_id
    }

    #[inline]
    pub fn set_title(&self, title: &str) {
        if let Err(e) = self
            .window_requests_tx
            .send((self.window_id, WindowRequest::Title(title.to_string())))
        {
            log::warn!("Fail to send title request: {}", e);
        }
    }

    #[inline]
    pub fn set_transparent(&self, transparent: bool) {
        todo!()
    }

    #[inline]
    pub fn set_visible(&self, visible: bool) {
        if let Err(e) = self
            .window_requests_tx
            .send((self.window_id, WindowRequest::Visible(visible)))
        {
            log::warn!("Fail to send visible request: {}", e);
        }
    }

    #[inline]
    pub fn is_visible(&self) -> Option<bool> {
        Some(self.window.is_visible())
    }

    #[inline]
    pub fn outer_position(&self) -> Result<PhysicalPosition<i32>, NotSupportedError> {
        let (x, y) = &*self.position;
        Ok(
            LogicalPosition::new(x.load(Ordering::Acquire), y.load(Ordering::Acquire))
                .to_physical(self.scale_factor.load(Ordering::Acquire) as f64),
        )
    }
    #[inline]
    pub fn inner_position(&self) -> Result<PhysicalPosition<i32>, NotSupportedError> {
        let (x, y) = &*self.position;
        Ok(
            LogicalPosition::new(x.load(Ordering::Acquire), y.load(Ordering::Acquire))
                .to_physical(self.scale_factor.load(Ordering::Acquire) as f64),
        )
    }
    #[inline]
    pub fn set_outer_position(&self, position: Position) {
        let (x, y): (i32, i32) = position.to_logical::<i32>(self.scale_factor()).into();

        if let Err(e) = self
            .window_requests_tx
            .send((self.window_id, WindowRequest::Position((x, y))))
        {
            log::warn!("Fail to send position request: {}", e);
        }
    }
    #[inline]
    pub fn inner_size(&self) -> PhysicalSize<u32> {
        let (width, height) = &*self.size;

        LogicalSize::new(
            width.load(Ordering::Acquire) as u32,
            height.load(Ordering::Acquire) as u32,
        )
        .to_physical(self.scale_factor.load(Ordering::Acquire) as f64)
    }

    #[inline]
    pub fn outer_size(&self) -> PhysicalSize<u32> {
        let (width, height) = &*self.size;

        LogicalSize::new(
            width.load(Ordering::Acquire) as u32,
            height.load(Ordering::Acquire) as u32,
        )
        .to_physical(self.scale_factor.load(Ordering::Acquire) as f64)
    }

    #[inline]
    pub fn set_inner_size(&self, size: Size) {
        let (width, height) = size.to_logical::<i32>(self.scale_factor()).into();

        if let Err(e) = self
            .window_requests_tx
            .send((self.window_id, WindowRequest::Size((width, height))))
        {
            log::warn!("Fail to send size request: {}", e);
        }
    }

    #[inline]
    pub fn set_min_inner_size(&self, dimensions: Option<Size>) {
        todo!()
    }

    #[inline]
    pub fn set_max_inner_size(&self, dimensions: Option<Size>) {
        todo!()
    }

    #[inline]
    pub fn resize_increments(&self) -> Option<PhysicalSize<u32>> {
        todo!()
    }

    #[inline]
    pub fn set_resize_increments(&self, increments: Option<Size>) {
        todo!()
    }

    #[inline]
    pub fn set_resizable(&self, resizable: bool) {
        todo!()
    }

    #[inline]
    pub fn is_resizable(&self) -> bool {
        todo!()
    }

    #[inline]
    pub fn set_enabled_buttons(&self, buttons: WindowButtons) {
        todo!()
    }

    #[inline]
    pub fn enabled_buttons(&self) -> WindowButtons {
        todo!()
    }

    #[inline]
    pub fn set_cursor_icon(&self, cursor: CursorIcon) {
        todo!()
    }

    #[inline]
    pub fn set_cursor_grab(&self, mode: CursorGrabMode) -> Result<(), ExternalError> {
        todo!()
    }

    #[inline]
    pub fn set_cursor_visible(&self, visible: bool) {
        todo!()
    }

    #[inline]
    pub fn drag_window(&self) -> Result<(), ExternalError> {
        todo!()
    }

    #[inline]
    pub fn drag_resize_window(&self, direction: ResizeDirection) -> Result<(), ExternalError> {
        todo!()
    }

    #[inline]
    pub fn set_cursor_hittest(&self, hittest: bool) -> Result<(), ExternalError> {
        todo!()
    }

    #[inline]
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor.load(Ordering::Acquire) as f64
    }

    #[inline]
    pub fn set_cursor_position(&self, position: Position) -> Result<(), ExternalError> {
        todo!()
    }

    #[inline]
    pub fn set_maximized(&self, maximized: bool) {
        todo!()
    }

    #[inline]
    pub fn is_maximized(&self) -> bool {
        todo!()
    }

    #[inline]
    pub fn set_minimized(&self, minimized: bool) {
        todo!()
    }

    #[inline]
    pub fn is_minimized(&self) -> Option<bool> {
        todo!()
    }

    #[inline]
    pub(crate) fn fullscreen(&self) -> Option<Fullscreen> {
        todo!()
    }

    #[inline]
    pub(crate) fn set_fullscreen(&self, monitor: Option<Fullscreen>) {
        todo!()
    }

    #[inline]
    pub fn set_decorations(&self, decorations: bool) {
        todo!()
    }

    #[inline]
    pub fn is_decorated(&self) -> bool {
        todo!()
    }

    #[inline]
    pub fn set_window_level(&self, level: WindowLevel) {
        todo!()
    }

    #[inline]
    pub fn set_window_icon(&self, window_icon: Option<Icon>) {
        todo!()
    }

    #[inline]
    pub fn set_ime_cursor_area(&self, position: Position, size: Size) {
        todo!()
    }

    #[inline]
    pub fn reset_dead_keys(&self) {
        todo!()
    }

    #[inline]
    pub fn set_ime_position(&self, position: Position) {
        todo!()
    }

    #[inline]
    pub fn set_ime_allowed(&self, allowed: bool) {
        todo!()
    }

    #[inline]
    pub fn set_ime_purpose(&self, purpose: ImePurpose) {
        todo!()
    }

    #[inline]
    pub fn focus_window(&self) {
        todo!()
    }

    pub fn request_user_attention(&self, request_type: Option<UserAttentionType>) {
        todo!()
    }

    #[inline]
    pub fn request_redraw(&self) {
        todo!()
    }

    #[inline]
    pub fn pre_present_notify(&self) {
        todo!()
    }

    #[inline]
    pub fn current_monitor(&self) -> Option<MonitorHandle> {
        todo!()
    }

    #[inline]
    pub fn available_monitors(&self) -> VecDeque<MonitorHandle> {
        todo!()
    }

    #[inline]
    pub fn primary_monitor(&self) -> Option<MonitorHandle> {
        todo!()
    }

    #[inline]
    pub fn raw_window_handle(&self) -> RawWindowHandle {
        todo!()
    }

    #[inline]
    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        todo!()
    }

    #[inline]
    pub fn set_theme(&self, theme: Option<Theme>) {
        todo!()
    }

    #[inline]
    pub fn theme(&self) -> Option<Theme> {
        todo!()
    }

    pub fn set_content_protected(&self, protected: bool) {
        todo!()
    }

    #[inline]
    pub fn has_focus(&self) -> bool {
        todo!()
    }

    pub fn title(&self) -> String {
        todo!()
    }
}
