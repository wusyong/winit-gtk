use gtk::traits::{GtkWindowExt, WidgetExt};

use crate::dpi::{LogicalSize, Size};

pub fn set_size_constraints<W: GtkWindowExt + WidgetExt>(
    window: &W,
    min_size: Option<Size>,
    max_size: Option<Size>,
) {
    let mut geom_mask = gdk::WindowHints::empty();
    if min_size.is_some() {
        geom_mask |= gdk::WindowHints::MIN_SIZE;
    }
    if max_size.is_some() {
        geom_mask |= gdk::WindowHints::MAX_SIZE;
    }

    let scale_factor = window.scale_factor() as f64;

    let min_size: LogicalSize<i32> = min_size
        .map(|s| s.to_logical(scale_factor))
        .unwrap_or(LogicalSize::new(0, 0));
    let max_size: LogicalSize<i32> = max_size
        .map(|s| s.to_logical(scale_factor))
        .unwrap_or(LogicalSize::new(i32::MAX, i32::MAX));

    let picky_none: Option<&gtk::Window> = None;
    window.set_geometry_hints(
        picky_none,
        Some(&gdk::Geometry::new(
            min_size.width,
            min_size.height,
            max_size.width,
            max_size.height,
            0,
            0,
            0,
            0,
            0f64,
            0f64,
            gdk::Gravity::Center,
        )),
        geom_mask,
    )
}
