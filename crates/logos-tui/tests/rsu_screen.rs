use logos_tui::{App, View};

#[test]
fn test_rsu_render() {
    let mut app = App::new();
    app.set_view(View::Rsu);
    let frame = app.render_frame();
    assert!(frame.contains("RSU View | 30-day Avg | Haircut Forecast"));
}
