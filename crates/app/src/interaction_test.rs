//! Real-window controller regressions without mouse injection or installed hooks.
use super::*;
pub fn run(ui: slint::Weak<Editor>, ring: slint::Weak<Ring>, state: Rc<RefCell<State>>, case: u32) {
    let (Some(editor), Some(overlay)) = (ui.upgrade(), ring.upgrade()) else {
        panic!("missing test windows")
    };
    if case == 14 {
        close(&mut state.borrow_mut(), &overlay);
        submenu_regression(&editor, &overlay, &mut state.borrow_mut());
        println!(
            "INTERACTION OK: five mouse buttons, keyboard triggers, delayed short releases, click mode and persistent overlay"
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
            submenu_hover: None,
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
        s.config.settings.keyboard_key = match case {
            10 | 11 => Some(0x7c),
            12 | 13 => Some(0x41),
            _ => None,
        };
        s.config.settings.hold = case < 5 || (case >= 10 && case.is_multiple_of(2));
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
                let glass = editor.get_menu_liquid_glass();
                let glazed = overlay.window().take_snapshot().unwrap();
                editor.set_menu_liquid_glass(!glass);
                preview_background(&editor, &overlay);
                let flat = overlay.window().take_snapshot().unwrap();
                assert_ne!(
                    glazed.as_bytes(),
                    flat.as_bytes(),
                    "glass toggle did not change menu rendering"
                );
                editor.set_menu_liquid_glass(glass);
                preview_background(&editor, &overlay);
                let before = overlay.window().take_snapshot().unwrap();
                let app_background = editor.get_background_hex();
                let app_opacity = editor.get_glass_strength();
                let app_glass = editor.get_app_liquid_glass();
                editor.set_app_liquid_glass(!app_glass);
                let app_accent_hex = editor.get_accent_hex();
                let app_light = editor.global::<Ink>().get_light();
                let app_accent = editor.global::<Ink>().get_accent();
                editor.set_background_hex("#3399FF".into());
                editor.set_glass_strength(15.0);
                editor
                    .global::<Ink>()
                    .set_light(!editor.global::<Ink>().get_light());
                editor.set_accent_hex("#FF3300".into());
                preview_background(&editor, &overlay);
                let after = overlay.window().take_snapshot().unwrap();
                assert_eq!(
                    before.as_bytes(),
                    after.as_bytes(),
                    "app colors or theme changed the radial menu"
                );
                editor.set_background_hex(app_background);
                editor.set_glass_strength(app_opacity);
                editor.set_app_liquid_glass(app_glass);
                editor.set_accent_hex(app_accent_hex);
                editor.global::<Ink>().set_light(app_light);
                editor.global::<Ink>().set_accent(app_accent);
                preview_background(&editor, &overlay);
                for (window, filename) in [
                    (editor.window(), "appearance-test.png"),
                    (overlay.window(), "ring-test.png"),
                ] {
                    let pixels = window.take_snapshot().unwrap();
                    if filename == "ring-test.png" {
                        let (width, height) = (pixels.width(), pixels.height());
                        let pixel =
                            pixels.as_slice()[((height * 30 / 100) * width + width / 2) as usize];
                        assert!(
                            pixel.a < 245 && pixel.a > 180,
                            "menu background opacity was clamped or lost: {}",
                            pixel.a
                        );

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
            if case == 0 {
                editor.set_page(4);
                let pixels = editor.window().take_snapshot().unwrap();
                image::save_buffer(
                    std::env::current_dir()
                        .unwrap()
                        .join("target/menu-colors-test.png"),
                    pixels.as_bytes(),
                    pixels.width(),
                    pixels.height(),
                    image::ColorType::Rgba8,
                )
                .unwrap();
                editor.set_icon_query("alarm".into());
                editor.set_icon_library(1);
                icon_catalog::refresh(&editor);
                editor.set_page(5);
                let icons = editor.window().take_snapshot().unwrap();
                image::save_buffer(
                    std::env::current_dir()
                        .unwrap()
                        .join("target/icon-library-test.png"),
                    icons.as_bytes(),
                    icons.width(),
                    icons.height(),
                    image::ColorType::Rgba8,
                )
                .unwrap();
                editor.set_icon_query("home".into());
                editor.set_icon_library(2);
                icon_catalog::refresh(&editor);
                let material = editor.window().take_snapshot().unwrap();
                image::save_buffer(
                    std::env::current_dir()
                        .unwrap()
                        .join("target/material-icons-test.png"),
                    material.as_bytes(),
                    material.width(),
                    material.height(),
                    image::ColorType::Rgba8,
                )
                .unwrap();
                editor.set_page(2);
            }
            if case < 5 || (case >= 10 && case.is_multiple_of(2)) {
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

fn submenu_regression(ui: &Editor, ring: &Ring, state: &mut State) {
    let center = native::cursor().unwrap();
    let child = "test-submenu".to_string();
    state.config.menus.push(Menu {
        id: child.clone(),
        name: "Child".into(),
        slots: Default::default(),
    });
    state.config.menus[0].slots[0].as_mut().unwrap().action = Action::Submenu { menu: child };
    state.config.settings.hold = true;
    state.config.settings.keyboard_key = Some(0x7c);
    for offset in [0.0, -112.0] {
        let at = Instant::now() - Duration::from_millis(350);
        process(
            state,
            ui,
            ring,
            Input::Press {
                point: center,
                foreground: native::foreground(),
                at,
            },
        );
        choose(state, ui, ring, Some(0));
        assert!(
            state.session.as_ref().unwrap().held,
            "submenu discarded held state"
        );
        assert_eq!(state.session.as_ref().unwrap().stack.len(), 2);
        let scale = state.session.as_ref().unwrap().scale;
        let released = Point {
            x: center.x,
            y: center.y + (offset * scale) as i32,
        };
        process(
            state,
            ui,
            ring,
            Input::Release {
                point: released,
                at: at + Duration::from_millis(300),
            },
        );
        assert!(!ring.window().is_visible(), "release left submenu open");
    }
    let at = Instant::now() - Duration::from_millis(350);
    process(
        state,
        ui,
        ring,
        Input::Press {
            point: center,
            foreground: native::foreground(),
            at,
        },
    );
    let scale = state.session.as_ref().unwrap().scale;
    process(
        state,
        ui,
        ring,
        Input::Release {
            point: Point {
                x: center.x,
                y: center.y - (112.0 * scale) as i32,
            },
            at: at + Duration::from_millis(300),
        },
    );
    assert!(
        !ring.window().is_visible(),
        "releasing on submenu target left it open"
    );
    state.config.settings.hold = false;
    process(
        state,
        ui,
        ring,
        Input::Press {
            point: center,
            foreground: native::foreground(),
            at: Instant::now(),
        },
    );
    choose(state, ui, ring, Some(0));
    process(
        state,
        ui,
        ring,
        Input::Release {
            point: center,
            at: Instant::now(),
        },
    );
    assert!(
        ring.window().is_visible(),
        "click navigation should stay available"
    );
    process(state, ui, ring, Input::Escape);
    println!("SUBMENU OK: hold release closes child/empty/back targets; click mode preserved");
}
