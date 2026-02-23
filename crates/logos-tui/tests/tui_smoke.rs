use logos_tui::App;

#[test]
fn tui_app_renders_one_frame_and_exits_cleanly() {
    let mut app = App::new();
    let frame = app.render_frame();

    assert!(frame.contains("Logos"));

    app.request_exit();
    assert!(app.should_exit());
}
