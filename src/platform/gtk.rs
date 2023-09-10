use crate::{
    event_loop::EventLoopWindowTarget,
    platform_impl::ApplicationName,
    window::{Window, WindowBuilder},
};

/// Additional methods on `Window` that are specific to Unix.
pub trait WindowExtUnix {
    /// Returns the `gtk::ApplicatonWindow` from gtk crate that is used by this window.
    fn gtk_window(&self) -> &gtk::ApplicationWindow;

    /// Returns the vertical `gtk::Box` that is added by default as the sole child of this window.
    /// Returns `None` if the default vertical `gtk::Box` creation was disabled by [`WindowBuilderExtUnix::with_default_vbox`].
    fn default_vbox(&self) -> Option<&gtk::Box>;

    /// Whether to show the window icon in the taskbar or not.
    fn set_skip_taskbar(&self, skip: bool);
}

impl WindowExtUnix for Window {
    fn gtk_window(&self) -> &gtk::ApplicationWindow {
        &self.window.window
    }

    fn default_vbox(&self) -> Option<&gtk::Box> {
        self.window.default_vbox.as_ref()
    }

    fn set_skip_taskbar(&self, skip: bool) {
        self.window.set_skip_taskbar(skip);
    }
}

pub trait WindowBuilderExtUnix {
    /// Build window with the given `general` and `instance` names.
    ///
    /// The `general` sets general class of `WM_CLASS(STRING)`, while `instance` set the
    /// instance part of it. The resulted property looks like `WM_CLASS(STRING) = "general", "instance"`.
    ///
    /// For details about application ID conventions, see the
    /// [Desktop Entry Spec](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html#desktop-file-id)
    fn with_name(self, general: impl Into<String>, instance: impl Into<String>) -> Self;

    /// Whether to create the window icon with the taskbar icon or not.
    fn with_skip_taskbar(self, skip: bool) -> WindowBuilder;

    /// Whether to enable or disable the internal draw for transparent window.
    ///
    /// When tranparent attribute is enabled, we will call `connect_draw` and draw a transparent background.
    /// For anyone who wants to draw the background themselves, set this to `false`.
    /// Default is `true`.
    fn with_transparent_draw(self, draw: bool) -> WindowBuilder;

    /// Whether to enable or disable the double buffered rendering of the window.
    ///
    /// Default is `true`.
    fn with_double_buffered(self, double_buffered: bool) -> WindowBuilder;

    /// Whether to enable the rgba visual for the window.
    ///
    /// Default is `false` but is always `true` if [`WindowAttributes::transparent`](crate::window::WindowAttributes::transparent) is `true`
    fn with_rgba_visual(self, rgba_visual: bool) -> WindowBuilder;

    /// Wether to set this window as app paintable
    ///
    /// <https://docs.gtk.org/gtk3/method.Widget.set_app_paintable.html>
    ///
    /// Default is `false` but is always `true` if [`WindowAttributes::transparent`](crate::window::WindowAttributes::transparent) is `true`
    fn with_app_paintable(self, app_paintable: bool) -> WindowBuilder;

    /// Whether to create a vertical `gtk::Box` and add it as the sole child of this window.
    /// Created by default.
    fn with_default_vbox(self, add: bool) -> WindowBuilder;
}

impl WindowBuilderExtUnix for WindowBuilder {
    fn with_name(mut self, general: impl Into<String>, instance: impl Into<String>) -> Self {
        // TODO We haven't implemented it yet.
        self.platform_specific.name = Some(ApplicationName::new(general.into(), instance.into()));
        self
    }
    fn with_skip_taskbar(mut self, skip: bool) -> WindowBuilder {
        self.platform_specific.skip_taskbar = skip;
        self
    }

    fn with_transparent_draw(mut self, draw: bool) -> WindowBuilder {
        self.platform_specific.auto_transparent = draw;
        self
    }

    fn with_double_buffered(mut self, double_buffered: bool) -> WindowBuilder {
        self.platform_specific.double_buffered = double_buffered;
        self
    }

    fn with_rgba_visual(mut self, rgba_visual: bool) -> WindowBuilder {
        self.platform_specific.rgba_visual = rgba_visual;
        self
    }

    fn with_app_paintable(mut self, app_paintable: bool) -> WindowBuilder {
        self.platform_specific.app_paintable = app_paintable;
        self
    }

    fn with_default_vbox(mut self, add: bool) -> WindowBuilder {
        self.platform_specific.default_vbox = add;
        self
    }
}

/// Additional methods on `EventLoopWindowTarget` that are specific to Unix.
pub trait EventLoopWindowTargetExtUnix {
    /// True if the `EventLoopWindowTarget` uses Wayland.
    fn is_wayland(&self) -> bool;
}

impl<T> EventLoopWindowTargetExtUnix for EventLoopWindowTarget<T> {
    #[inline]
    fn is_wayland(&self) -> bool {
        self.p.is_wayland()
    }
}
