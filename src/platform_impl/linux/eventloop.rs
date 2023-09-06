use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
    rc::Rc,
};

use crossbeam_channel::SendError;
use gdk::prelude::{ApplicationExt, DisplayExtManual};
use gio::Cancellable;
use glib::{MainContext, ObjectType, Priority};
use raw_window_handle::{RawDisplayHandle, WaylandDisplayHandle, XlibDisplayHandle};

use crate::{
    event::{Event, StartCause},
    event_loop::{
        ControlFlow, DeviceEventFilter, EventLoopClosed, EventLoopWindowTarget as RootELW,
    },
};

use super::{
    monitor::MonitorHandle, window::WindowRequest, PlatformSpecificEventLoopAttributes, WindowId,
};

pub struct EventLoop<T: 'static> {
    /// Window target.
    window_target: RootELW<T>,
    /// User event sender for EventLoopProxy
    pub(crate) user_event_tx: crossbeam_channel::Sender<Event<'static, T>>,
    /// Event queue of EventLoop
    events: crossbeam_channel::Receiver<Event<'static, T>>,
    /// Draw queue of EventLoop
    draws: crossbeam_channel::Receiver<WindowId>,
}

/// Used to send custom events to `EventLoop`.
#[derive(Debug)]
pub struct EventLoopProxy<T: 'static> {
    user_event_tx: crossbeam_channel::Sender<Event<'static, T>>,
}

impl<T: 'static> Clone for EventLoopProxy<T> {
    fn clone(&self) -> Self {
        Self {
            user_event_tx: self.user_event_tx.clone(),
        }
    }
}

impl<T: 'static> EventLoop<T> {
    pub(crate) fn new(attributes: &PlatformSpecificEventLoopAttributes) -> Self {
        let context = MainContext::default();
        let app = gtk::Application::new(None, gio::ApplicationFlags::empty());
        let app_ = app.clone();
        let cancellable: Option<&Cancellable> = None;
        app.register(cancellable)
            .expect("Failed to register GtkApplication");

        // Create channels for handling events and send StartCause::Init event
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        let (draw_tx, draw_rx) = crossbeam_channel::unbounded();
        let event_tx_ = event_tx.clone();
        let draw_tx_ = draw_tx.clone();
        let user_event_tx = event_tx.clone();
        app.connect_activate(move |_| {
            if let Err(e) = event_tx_.send(Event::NewEvents(StartCause::Init)) {
                log::warn!("Failed to send init event to event channel: {}", e);
            }
        });

        // Create event loop window target.
        let (window_requests_tx, window_requests_rx) =
            glib::MainContext::channel(Priority::default());
        let display = gdk::Display::default()
            .expect("GdkDisplay not found. This usually means `gkt_init` hasn't called yet.");
        let window_target = EventLoopWindowTarget {
            display,
            app,
            windows: Rc::new(RefCell::new(HashSet::new())),
            window_requests_tx,
            draw_tx: draw_tx_,
            _marker: std::marker::PhantomData,
        };

        // TODO: Spawn x11/wayland thread to receive Device events.

        // TODO: Handle shit tons of window events.

        // Create event loop itself.
        let event_loop = Self {
            window_target: RootELW {
                p: window_target,
                _marker: std::marker::PhantomData,
            },
            user_event_tx,
            events: event_rx,
            draws: draw_rx,
        };

        event_loop
    }
    /// Creates an `EventLoopProxy` that can be used to dispatch user events to the main event loop.
    pub fn create_proxy(&self) -> EventLoopProxy<T> {
        EventLoopProxy {
            user_event_tx: self.user_event_tx.clone(),
        }
    }

    pub fn run<F>(mut self, callback: F) -> !
    where
        F: 'static + FnMut(crate::event::Event<'_, T>, &RootELW<T>, &mut ControlFlow),
    {
        todo!()
    }

    pub fn window_target(&self) -> &crate::event_loop::EventLoopWindowTarget<T> {
        &self.window_target
    }
}

impl<T: 'static> EventLoopProxy<T> {
    pub fn send_event(&self, event: T) -> Result<(), EventLoopClosed<T>> {
        self.user_event_tx
            .send(Event::UserEvent(event))
            .map_err(|SendError(event)| {
                if let Event::UserEvent(error) = event {
                    EventLoopClosed(error)
                } else {
                    unreachable!();
                }
            })?;

        let context = MainContext::default();
        context.wakeup();

        Ok(())
    }
}

#[derive(Clone)]
pub struct EventLoopWindowTarget<T> {
    /// Gdk display
    pub(crate) display: gdk::Display,
    /// Gtk application
    pub(crate) app: gtk::Application,
    /// Window Ids of the application
    pub(crate) windows: Rc<RefCell<HashSet<WindowId>>>,
    /// Window requests sender
    pub(crate) window_requests_tx: glib::Sender<(WindowId, WindowRequest)>,
    /// Draw event sender
    pub(crate) draw_tx: crossbeam_channel::Sender<WindowId>,
    _marker: std::marker::PhantomData<T>,
}
impl<T> EventLoopWindowTarget<T> {
    #[inline]
    pub fn is_wayland(&self) -> bool {
        self.display.backend().is_wayland()
    }

    #[inline]
    pub fn available_monitors(&self) -> VecDeque<MonitorHandle> {
        let mut handles = VecDeque::new();
        let display = &self.display;
        let numbers = display.n_monitors();

        for i in 0..numbers {
            let monitor = MonitorHandle::new(display, i);
            handles.push_back(monitor);
        }

        handles
    }

    #[inline]
    pub fn primary_monitor(&self) -> Option<MonitorHandle> {
        let monitor = self.display.primary_monitor();
        monitor.and_then(|monitor| {
            let handle = MonitorHandle { monitor };
            Some(handle)
        })
    }

    #[inline]
    pub fn set_device_event_filter(&self, _filter: DeviceEventFilter) {
        todo!()
    }

    pub fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        if self.is_wayland() {
            let mut display_handle = WaylandDisplayHandle::empty();
            display_handle.display = unsafe {
                gdk_wayland_sys::gdk_wayland_display_get_wl_display(self.display.as_ptr() as *mut _)
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
}
