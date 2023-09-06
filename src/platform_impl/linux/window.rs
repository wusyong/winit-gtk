use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    sync::atomic::{AtomicBool, AtomicI32, Ordering},
};

use gdk::{prelude::DisplayExtManual, WindowEdge, WindowState};
use glib::{translate::ToGlibPtr, Cast, ObjectType};
use gtk::{
    traits::{ApplicationWindowExt, ContainerExt, GtkWindowExt, SettingsExt, WidgetExt},
    Inhibit, Settings,
};
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
    XlibDisplayHandle, XlibWindowHandle,
};

use crate::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size},
    error::{ExternalError, NotSupportedError, OsError as RootOsError},
    platform_impl::WindowId,
    window::{
        CursorGrabMode, CursorIcon, Fullscreen as RootFullscreen, Icon, ImePurpose,
        ResizeDirection, Theme, UserAttentionType, WindowAttributes, WindowButtons, WindowLevel,
    },
};

use super::{
    EventLoopWindowTarget, Fullscreen, MonitorHandle, PlatformSpecificWindowBuilderAttributes,
};

// Currently GTK doesn't provide feature for detect theme, so we need to check theme manually.
// ref: https://github.com/WebKit/WebKit/blob/e44ffaa0d999a9807f76f1805943eea204cfdfbc/Source/WebKit/UIProcess/API/gtk/PageClientImpl.cpp#L587
const GTK_THEME_SUFFIX_LIST: [&'static str; 3] = ["-dark", "-Dark", "-Darker"];

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
    fullscreen: RefCell<Option<RootFullscreen>>,
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
        let app = &window_target.app;
        let window_requests_tx = window_target.window_requests_tx.clone();
        let draw_tx = window_target.draw_tx.clone();
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .accept_focus(attribs.active)
            .build();
        let window_id = WindowId(window.id() as u64);
        window_target.windows.borrow_mut().insert(window_id);

        // Set Width/Height & Resizable
        let win_scale_factor = window.scale_factor();
        let (width, height) = attribs
            .inner_size
            .map(|size| size.to_logical::<f64>(win_scale_factor as f64).into())
            .unwrap_or((800, 600));
        window.set_default_size(1, 1);
        window.resize(width, height);

        if attribs.maximized {
            window.maximize();
        }
        window.set_resizable(attribs.resizable);
        // window.set_deletable(attribs.closable);

        // TODO Set Min/Max Size
        // util::set_size_constraints(&window, attribs.inner_size_constraints);

        // Set Position
        if let Some(position) = attribs.position {
            let (x, y): (i32, i32) = position.to_logical::<i32>(win_scale_factor as f64).into();
            window.move_(x, y);
        }

        // Set GDK Visual
        if pl_attribs.rgba_visual || attribs.transparent {
            if let Some(screen) = GtkWindowExt::screen(&window) {
                if let Some(visual) = screen.rgba_visual() {
                    window.set_visual(Some(&visual));
                }
            }
        }

        if pl_attribs.app_paintable || attribs.transparent {
            // Set a few attributes to make the window can be painted.
            // See Gtk drawing model for more info:
            // https://docs.gtk.org/gtk3/drawing-model.html
            window.set_app_paintable(true);
        }

        if !pl_attribs.double_buffered {
            let widget = window.upcast_ref::<gtk::Widget>();
            if !window_target.is_wayland() {
                unsafe {
                    gtk::ffi::gtk_widget_set_double_buffered(widget.to_glib_none().0, 0);
                }
            }
        }

        // Add vbox for utilities like menu
        let default_vbox = if pl_attribs.default_vbox {
            let box_ = gtk::Box::new(gtk::Orientation::Vertical, 0);
            window.add(&box_);
            Some(box_)
        } else {
            None
        };

        // Rest attributes
        window.set_title(&attribs.title);
        if let Some(RootFullscreen::Borderless(m)) = &attribs.fullscreen {
            if let Some(monitor) = m {
                let display = window.display();
                let monitor = &monitor.inner;
                let monitors = display.n_monitors();
                for i in 0..monitors {
                    let m = display.monitor(i).unwrap();
                    if m == monitor.monitor {
                        let screen = display.default_screen();
                        window.fullscreen_on_monitor(&screen, i);
                    }
                }
            } else {
                window.fullscreen();
            }
        }
        window.set_visible(attribs.visible);
        window.set_decorated(attribs.decorations);

        match attribs.window_level {
            WindowLevel::AlwaysOnBottom => window.set_keep_below(true),
            WindowLevel::Normal => (),
            WindowLevel::AlwaysOnTop => window.set_keep_above(true),
        }

        // TODO imeplement this?
        // if attribs.visible_on_all_workspaces {
        //     window.stick();
        // }

        if let Some(icon) = attribs.window_icon {
            window.set_icon(Some(&icon.inner.into()));
        }

        // Set theme
        let settings = Settings::default();

        if let Some(settings) = settings {
            if let Some(preferred_theme) = attribs.preferred_theme {
                match preferred_theme {
                    Theme::Dark => settings.set_gtk_application_prefer_dark_theme(true),
                    Theme::Light => {
                        let theme_name = settings.gtk_theme_name().map(|t| t.as_str().to_owned());
                        if let Some(theme) = theme_name {
                            // Remove dark variant.
                            if let Some(theme) = GTK_THEME_SUFFIX_LIST
                                .iter()
                                .find(|t| theme.ends_with(*t))
                                .map(|v| theme.strip_suffix(v))
                            {
                                settings.set_gtk_theme_name(theme);
                            }
                        }
                    }
                }
            }
        }

        if attribs.visible {
            window.show_all();
        } else {
            window.hide();
        }

        // TODO add parent window
        // if let Parent::ChildOf(parent) = pl_attribs.parent {
        //     window.set_transient_for(Some(&parent));
        // }

        // TODO I don't understand why unfocussed window need focus
        // restore accept-focus after the window has been drawn
        // if the window was initially created without focus
        // if !attributes.focused {
        //     let signal_id = Arc::new(RefCell::new(None));
        //     let signal_id_ = signal_id.clone();
        //     let id = window.connect_draw(move |window, _| {
        //         if let Some(id) = signal_id_.take() {
        //             window.set_accept_focus(true);
        //             window.disconnect(id);
        //         }
        //         Inhibit(false)
        //     });
        //     signal_id.borrow_mut().replace(id);
        // }

        // Set window position and size callback
        let w_pos = window.position();
        let position: Rc<(AtomicI32, AtomicI32)> = Rc::new((w_pos.0.into(), w_pos.1.into()));
        let position_clone = position.clone();

        let w_size = window.size();
        let size: Rc<(AtomicI32, AtomicI32)> = Rc::new((w_size.0.into(), w_size.1.into()));
        let size_clone = size.clone();

        window.connect_configure_event(move |_, event| {
            let (x, y) = event.position();
            position_clone.0.store(x, Ordering::Release);
            position_clone.1.store(y, Ordering::Release);

            let (w, h) = event.size();
            size_clone.0.store(w as i32, Ordering::Release);
            size_clone.1.store(h as i32, Ordering::Release);

            false
        });

        // Set minimized/maximized callback
        let w_max = window.is_maximized();
        let maximized: Rc<AtomicBool> = Rc::new(w_max.into());
        let max_clone = maximized.clone();
        let minimized = Rc::new(AtomicBool::new(false));
        let minimized_clone = minimized.clone();

        window.connect_window_state_event(move |_, event| {
            let state = event.new_window_state();
            max_clone.store(state.contains(WindowState::MAXIMIZED), Ordering::Release);
            minimized_clone.store(state.contains(WindowState::ICONIFIED), Ordering::Release);
            Inhibit(false)
        });

        // Set scale factor callback
        let scale_factor: Rc<AtomicI32> = Rc::new(win_scale_factor.into());
        let scale_factor_clone = scale_factor.clone();
        window.connect_scale_factor_notify(move |window| {
            scale_factor_clone.store(window.scale_factor(), Ordering::Release);
        });

        // Check if we should paint the transparent background ourselves.
        let mut transparent = false;
        if attribs.transparent && pl_attribs.auto_transparent {
            transparent = true;
        }

        // Send WireUp event to let eventloop handle the rest of window setup to prevent gtk panic
        // in other thread.
        let cursor_moved = pl_attribs.cursor_moved;
        if let Err(e) = window_requests_tx.send((
            window_id,
            WindowRequest::WireUpEvents {
                transparent,
                cursor_moved,
            },
        )) {
            log::warn!("Fail to send wire up events request: {}", e);
        }

        if let Err(e) = draw_tx.send(window_id) {
            log::warn!("Failed to send redraw event to event channel: {}", e);
        }

        let win = Self {
            window_id,
            window,
            default_vbox,
            window_requests_tx,
            draw_tx,
            scale_factor,
            position,
            size,
            maximized,
            minimized,
            fullscreen: RefCell::new(attribs.fullscreen),
            // inner_size_constraints: RefCell::new(attribs.inner_size_constraints),
        };

        // TODO
        // win.set_skip_taskbar(pl_attribs.skip_taskbar);

        Ok(win)
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
        // todo!()
        None
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
        if let Err(e) = self.draw_tx.send(self.window_id) {
            log::warn!("Failed to send redraw event to event channel: {}", e);
        }
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

    fn is_wayland(&self) -> bool {
        self.window.display().backend().is_wayland()
    }

    #[inline]
    pub fn raw_window_handle(&self) -> RawWindowHandle {
        if self.is_wayland() {
            let mut window_handle = WaylandWindowHandle::empty();
            if let Some(window) = self.window.window() {
                window_handle.surface = unsafe {
                    gdk_wayland_sys::gdk_wayland_window_get_wl_surface(window.as_ptr() as *mut _)
                };
            }

            RawWindowHandle::Wayland(window_handle)
        } else {
            let mut window_handle = XlibWindowHandle::empty();
            unsafe {
                if let Some(window) = self.window.window() {
                    window_handle.window =
                        gdk_x11_sys::gdk_x11_window_get_xid(window.as_ptr() as *mut _);
                }
            }
            RawWindowHandle::Xlib(window_handle)
        }
    }

    #[inline]
    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        if self.is_wayland() {
            let mut display_handle = WaylandDisplayHandle::empty();
            display_handle.display = unsafe {
                gdk_wayland_sys::gdk_wayland_display_get_wl_display(
                    self.window.display().as_ptr() as *mut _
                )
            };
            RawDisplayHandle::Wayland(display_handle)
        } else {
            let mut display_handle = XlibDisplayHandle::empty();
            unsafe {
                if let Ok(xlib) = x11_dl::xlib::Xlib::open() {
                    let display = (xlib.XOpenDisplay)(std::ptr::null());
                    display_handle.display = display as _;
                    display_handle.screen = (xlib.XDefaultScreen)(display) as _;
                }
            }

            RawDisplayHandle::Xlib(display_handle)
        }
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

/// A constant used to determine how much inside the window, the resize handler should appear (only used in Linux(gtk) and Windows).
/// You probably need to scale it by the scale_factor of the window.
pub const BORDERLESS_RESIZE_INSET: i32 = 5;

pub fn hit_test(window: &gdk::Window, cx: f64, cy: f64) -> WindowEdge {
    let (left, top) = window.position();
    let (w, h) = (window.width(), window.height());
    let (right, bottom) = (left + w, top + h);
    let (cx, cy) = (cx as i32, cy as i32);

    const LEFT: i32 = 0b0001;
    const RIGHT: i32 = 0b0010;
    const TOP: i32 = 0b0100;
    const BOTTOM: i32 = 0b1000;
    const TOPLEFT: i32 = TOP | LEFT;
    const TOPRIGHT: i32 = TOP | RIGHT;
    const BOTTOMLEFT: i32 = BOTTOM | LEFT;
    const BOTTOMRIGHT: i32 = BOTTOM | RIGHT;

    let inset = BORDERLESS_RESIZE_INSET * window.scale_factor();
    #[rustfmt::skip]
  let result =
      (LEFT * (if cx < (left + inset) { 1 } else { 0 }))
    | (RIGHT * (if cx >= (right - inset) { 1 } else { 0 }))
    | (TOP * (if cy < (top + inset) { 1 } else { 0 }))
    | (BOTTOM * (if cy >= (bottom - inset) { 1 } else { 0 }));

    match result {
        LEFT => WindowEdge::West,
        TOP => WindowEdge::North,
        RIGHT => WindowEdge::East,
        BOTTOM => WindowEdge::South,
        TOPLEFT => WindowEdge::NorthWest,
        TOPRIGHT => WindowEdge::NorthEast,
        BOTTOMLEFT => WindowEdge::SouthWest,
        BOTTOMRIGHT => WindowEdge::SouthEast,
        // we return `WindowEdge::__Unknown` to be ignored later.
        // we must return 8 or bigger, otherwise it will be the same as one of the other 7 variants of `WindowEdge` enum.
        _ => WindowEdge::__Unknown(8),
    }
}
