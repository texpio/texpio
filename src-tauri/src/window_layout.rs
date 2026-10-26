#[derive(Debug, PartialEq, Eq)]
pub(crate) struct StartupBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
}

pub(crate) fn startup_bounds(
    screen: (u32, u32),
    work: (i32, i32, u32, u32),
    scale: f64,
) -> Option<StartupBounds> {
    let (screen_width, screen_height) = screen;
    let (x, y, work_width, work_height) = work;
    if screen_width == 0
        || screen_height == 0
        || work_width == 0
        || work_height == 0
        || !scale.is_finite()
        || scale <= 0.0
    {
        return None;
    }
    let ratio = f64::from(screen_width) / f64::from(screen_height);
    let max_height = f64::from(work_height).min(f64::from(work_width) / ratio);
    let height = (max_height * 0.8)
        .max(560.0 * scale)
        .max(800.0 * scale / ratio)
        .min(max_height);
    let width = ((height * ratio).round() as u32).min(work_width);
    let height = (height.round() as u32).min(work_height);
    if width == 0 || height == 0 {
        return None;
    }
    Some(StartupBounds {
        x: (i64::from(x) + i64::from((work_width - width) / 2))
            .try_into()
            .ok()?,
        y: (i64::from(y) + i64::from((work_height - height) / 2))
            .try_into()
            .ok()?,
        width,
        height,
        min_width: ((800.0 * scale).ceil() as u32).min(width),
        min_height: ((560.0 * scale).ceil() as u32).min(height),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_show_finalizes_frame_before_fit_and_later_shows_preserve_user_bounds() {
        let layout = InitialWindowLayout::default();
        let calls = std::cell::RefCell::new(vec![]);
        layout
            .show_then_fit_once(
                || {
                    calls.borrow_mut().push("show");
                    Ok::<_, ()>(())
                },
                || {
                    assert_eq!(*calls.borrow(), vec!["show"]);
                    calls.borrow_mut().push("fit");
                },
            )
            .unwrap();
        layout
            .show_then_fit_once(
                || {
                    calls.borrow_mut().push("show");
                    Ok::<_, ()>(())
                },
                || panic!("reopening must preserve manual bounds"),
            )
            .unwrap();
        assert_eq!(*calls.borrow(), vec!["show", "fit", "show"]);
    }
    #[test]
    fn failed_show_does_not_size_or_consume_the_first_successful_show() {
        let layout = InitialWindowLayout::default();
        assert_eq!(
            layout.show_then_fit_once(|| Err(()), || panic!("hidden frame is not ready")),
            Err(())
        );
        let fitted = std::cell::Cell::new(false);
        layout
            .show_then_fit_once(|| Ok::<_, ()>(()), || fitted.set(true))
            .unwrap();
        assert!(fitted.get());
    }
    #[test]
    fn follows_screen_ratio_and_centers_inside_the_work_area() {
        let bounds = startup_bounds((1920, 1080), (0, 0, 1920, 1040), 1.0).unwrap();
        assert_eq!((bounds.width, bounds.height), (1479, 832));
        assert_eq!((bounds.x, bounds.y), (220, 104));
        assert_eq!((bounds.min_width, bounds.min_height), (800, 560));
        let portrait = startup_bounds((1080, 1920), (-1080, 40, 1080, 1880), 1.0).unwrap();
        assert_eq!((portrait.width, portrait.height), (846, 1504));
        assert_eq!((portrait.x, portrait.y), (-963, 228));
    }
    #[test]
    fn grows_for_logical_minimums_but_never_exceeds_small_or_scaled_work_areas() {
        let minimum = startup_bounds((1366, 768), (0, 0, 1366, 728), 1.25).unwrap();
        assert_eq!((minimum.width, minimum.height), (1245, 700));
        let small = startup_bounds((1024, 768), (-1024, 0, 1024, 728), 2.0).unwrap();
        assert_eq!((small.width, small.height), (971, 728));
        assert_eq!((small.min_width, small.min_height), (971, 728));
        assert_eq!((small.x, small.y), (-998, 0));
    }
    #[test]
    fn invalid_monitor_dimensions_or_scale_refuse_without_guessing() {
        for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(startup_bounds((1920, 1080), (0, 0, 1920, 1040), scale).is_none());
        }
        assert!(startup_bounds((0, 1080), (0, 0, 1920, 1040), 1.0).is_none());
        assert!(startup_bounds((1920, 1080), (0, 0, 1920, 0), 1.0).is_none());
    }
}
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub(crate) struct InitialWindowLayout {
    applied: AtomicBool,
}
impl InitialWindowLayout {
    pub(crate) fn show_then_fit_once<E>(
        &self,
        show: impl FnOnce() -> Result<(), E>,
        fit: impl FnOnce(),
    ) -> Result<(), E> {
        // Windows finalizes frameless border metrics when the window is shown.
        show()?;
        if !self.applied.swap(true, Ordering::AcqRel) {
            fit();
        }
        Ok(())
    }
}
