use framewell::{Editor, Tool, step::*};
use image::{Rgba, RgbaImage};
#[test]
fn markers_center_on_click_and_undo_without_drag_geometry() {
    let p = StepPreset::default();
    let mut e = Editor::new(RgbaImage::new(200, 200));
    e.add(p.marker([100., 80.], "1".into()));
    e.add(p.marker([160., 120.], "2".into()));
    assert_eq!(e.state.document.objects[0].bounds(), [72., 52., 56., 56.]);
    e.undo();
    assert_eq!(e.state.document.objects.len(), 1);
    e.redo();
    assert_eq!(e.state.document.objects[1].text, "2");
}
#[test]
fn alphabetic_sequence_rolls_past_z() {
    assert_eq!(StepSequence::Uppercase.label(26), "Z");
    assert_eq!(StepSequence::Uppercase.label(27), "AA");
    assert_eq!(StepSequence::Lowercase.label(703), "aaa");
    assert_eq!(StepSequence::Numbers.label(42), "42");
    for sequence in [
        StepSequence::Numbers,
        StepSequence::Uppercase,
        StepSequence::Lowercase,
    ] {
        for value in [1, 26, 27, 703, 9999] {
            assert_eq!(sequence.value(&sequence.label(value)), Some(value));
        }
        assert_eq!(sequence.value("!"), None);
    }
}
#[test]
fn step_shape_border_and_fill_reach_export_pixels() {
    let mut p = StepPreset::default();
    p.style.shape = StepShape::Square;
    p.style.border_color = [240, 10, 10, 255];
    p.fill = [10, 120, 20, 255];
    p.style.border_width = 4.;
    let mut e = Editor::new(RgbaImage::from_pixel(120, 120, Rgba([255, 255, 255, 255])));
    e.add(p.marker([60., 60.], String::new()));
    let img = e.render();
    assert_eq!(img.get_pixel(60, 60).0, p.fill);
    assert_eq!(img.get_pixel(33, 60).0, p.style.border_color);
    assert_eq!(img.get_pixel(20, 20).0, [255, 255, 255, 255]);
}
#[test]
fn legacy_steps_load_with_no_new_style_fields() {
    let json = r#"{"id":1,"tool":"Step","a":[10,10],"b":[50,50],"color":[0,100,100,255],"width":4,"text":"1","points":[]}"#;
    let o: framewell::Object = serde_json::from_str(json).unwrap();
    assert!(o.step.is_none());
    assert_eq!(o.tool, Tool::Step);
    assert!(o.hit([30., 30.]));
    assert!(!o.hit([5., 5.]));
}
#[test]
fn step_style_round_trip_preserves_export() {
    let mut e = Editor::new(RgbaImage::from_pixel(180, 150, Rgba([255, 255, 255, 255])));
    let mut p = StepPreset::default();
    p.style.shape = StepShape::Diamond;
    p.style.text_color = [240, 220, 10, 255];
    e.add(p.marker([90., 75.], "AB".into()));
    let dir = std::env::temp_dir().join(format!("framewell-step-test-{}", framewell::nonce()));
    e.save(&dir).unwrap();
    let r = Editor::open(&dir).unwrap();
    assert_eq!(e.render(), r.render());
    assert_eq!(
        r.state.document.objects[0].step,
        p.marker([0., 0.], "".into()).step
    );
    std::fs::remove_dir_all(dir).unwrap();
}
