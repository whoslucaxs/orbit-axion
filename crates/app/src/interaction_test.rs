//! Real-window controller regressions without mouse injection or installed hooks.
use super::*;
pub fn run(ui: slint::Weak<Editor>, ring: slint::Weak<Ring>, state: Rc<RefCell<State>>, case: u32) {
    let (Some(editor), Some(overlay)) = (ui.upgrade(), ring.upgrade()) else {
        panic!("missing test windows")
    };
    if case == 10 {
        close(&mut state.borrow_mut(), &overlay);
        println!(
            "INTERACTION OK: five buttons, delayed short releases, click mode and persistent overlay"
        );
        let _ = slint::quit_event_loop();
        return;
    }
    let center = native::cursor().unwrap();
    let origin = native::foreground();
    {
        let mut s = state.borrow_mut();
        let (hooks, events) = Hooks::detached_for_test();
        s.session = Some(Session {
            hooks,
            events,
            press: Instant::now(),
            gesture: Gesture::default(),
            wheel_slot: None,
            center,
            scale: 1.0,
            held: false,
            selected: None,
            last_read: Instant::now(),
            profile: "global".into(),
            stack: vec![],
            slots: Default::default(),
            query: String::new(),
            origin,
        });
        s.config.settings.mouse_button = case % 5;
        s.config.settings.hold = case < 5;
        s.config.settings.hold_ms = 220;
        let press = Instant::now() - Duration::from_millis(500);
        process(
            &mut s,
            &editor,
            &overlay,
            Input::Press {
                point: center,
                foreground: origin,
                at: press,
            },
        );
        process(
            &mut s,
            &editor,
            &overlay,
            Input::Release {
                point: center,
                at: press + Duration::from_millis(30),
            },
        );
        assert!(
            s.session
                .as_ref()
                .unwrap()
                .hooks
                .shared
                .active
                .load(Ordering::Acquire),
            "short click closed the ring with UI delay"
        );
        assert!(overlay.window().is_visible());
    }
    slint::Timer::single_shot(Duration::from_millis(120), move || {
        let (Some(editor), Some(overlay)) = (ui.upgrade(), ring.upgrade()) else {
            panic!("missing test windows")
        };
        {
            let mut s = state.borrow_mut();
            assert!(
                s.session
                    .as_ref()
                    .unwrap()
                    .hooks
                    .shared
                    .active
                    .load(Ordering::Acquire),
                "ring flashed and closed in case {case}; origin={origin:x}, foreground={:x}, ring={:x}, ready={}, paused={}",
                native::foreground(),
                s.hwnd.unwrap_or_default(),
                s.session
                    .as_ref()
                    .unwrap()
                    .hooks
                    .shared
                    .ready
                    .load(Ordering::Acquire),
                s.paused
            );
            assert!(overlay.window().is_visible());
            assert_eq!(native::foreground(), origin, "ring stole focus");
            if case == 0 {
                for (window, filename) in [
                    (editor.window(), "appearance-test.png"),
                    (overlay.window(), "ring-test.png"),
                ] {
                    let pixels = window.take_snapshot().unwrap();
                    if filename == "ring-test.png" {
                        let (width, height) = (pixels.width(), pixels.height());
                        for (x, y) in [
                            (0, 0),
                            (width - 1, 0),
                            (0, height - 1),
                            (width - 1, height - 1),
                        ] {
                            assert_eq!(
                                pixels.as_slice()[(y * width + x) as usize].a,
                                0,
                                "ring corners must be transparent"
                            );
                        }
                    }
                    if filename == "appearance-test.png" {
                        let pixel = pixels.as_slice()[((pixels.height() - 10) * pixels.width()
                            + pixels.width()
                            - 10) as usize];
                        assert!(
                            (pixel.a as i32 - 94).abs() <= 1,
                            "panel opacity did not reach the renderer"
                        );
                        // Premultiplication round-trips differ slightly between GPU/software.
                        for (actual, expected) in [(pixel.r, 70i32), (pixel.g, 69), (pixel.b, 69)] {
                            assert!(
                                (actual as i32 - expected).abs() <= 3,
                                "panel background color did not reach the renderer"
                            );
                        }
                    }
                    image::save_buffer(
                        std::env::current_dir()
                            .unwrap()
                            .join("target")
                            .join(filename),
                        pixels.as_bytes(),
                        pixels.width(),
                        pixels.height(),
                        image::ColorType::Rgba8,
                    )
                    .unwrap();
                }
            }
            if case < 5 {
                process(&mut s, &editor, &overlay, Input::Escape);
                let press = Instant::now() - Duration::from_millis(350);
                process(
                    &mut s,
                    &editor,
                    &overlay,
                    Input::Press {
                        point: center,
                        foreground: origin,
                        at: press,
                    },
                );
                process(
                    &mut s,
                    &editor,
                    &overlay,
                    Input::Release {
                        point: center,
                        at: press + Duration::from_millis(300),
                    },
                );
                assert!(
                    !s.session
                        .as_ref()
                        .unwrap()
                        .hooks
                        .shared
                        .active
                        .load(Ordering::Acquire),
                    "long hold at center did not cancel"
                );
            } else {
                process(&mut s, &editor, &overlay, Input::Escape);
                assert!(!overlay.window().is_visible());
            }
        }
        run(ui, ring, state, case + 1);
    });
}
