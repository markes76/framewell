use framewell::*;
use image::{Rgba, RgbaImage};
fn object(tool: Tool) -> Object {
    Object {
        id: 1,
        tool,
        a: [10., 10.],
        b: [40., 40.],
        color: [250, 50, 50, 255],
        width: 4.,
        text: "Secret".into(),
        points: vec![],
        step: None,
    }
}
#[test]
fn edits_undo_without_copying_source() {
    let mut e = Editor::new(sample());
    let original = e.state.pixels.clone();
    e.add(object(Tool::Arrow));
    assert!(std::sync::Arc::ptr_eq(&original, &e.state.pixels));
    e.undo();
    assert!(e.state.document.objects.is_empty());
    e.redo();
    assert_eq!(e.state.document.objects.len(), 1);
}
#[test]
fn privacy_masks_win_over_all_layers() {
    let source = RgbaImage::from_pixel(60, 60, Rgba([255, 255, 255, 255]));
    let image = render(
        &source,
        &[
            object(Tool::Redact),
            object(Tool::Text),
            object(Tool::Highlight),
            object(Tool::Blur),
        ],
    );
    for y in 10..40 {
        for x in 10..40 {
            assert_eq!(image.get_pixel(x, y), &Rgba([18, 24, 30, 255]));
        }
    }
    assert_eq!(source.get_pixel(20, 20), &Rgba([255, 255, 255, 255]));
}
#[test]
fn save_reopen_preserves_objects_and_source() {
    let mut e = Editor::new(sample());
    e.add(object(Tool::Callout));
    e.state.document.caption = "Explain the control".into();
    let p = std::env::temp_dir().join(format!("framewell-test-{}", nonce()));
    e.save(&p).unwrap();
    let restored = Editor::open(&p).unwrap();
    assert_eq!(restored.state.document.objects[0].text, "Secret");
    assert_eq!(restored.render(), e.render());
    e.save(&p).unwrap();
    assert!(Editor::open(&p).is_ok());
    std::fs::remove_dir_all(p).unwrap();
}
#[test]
fn invalid_geometry_is_rejected() {
    let mut d = Document::default();
    let mut o = object(Tool::Arrow);
    o.width = f32::NAN;
    d.objects.push(o);
    assert!(validate(&d, 100, 100).is_err());
}
#[test]
fn project_asset_traversal_is_rejected() {
    let p = std::env::temp_dir().join(format!("framewell-test-{}", nonce()));
    std::fs::create_dir(&p).unwrap();
    let v = serde_json::json!({"document":Document::default(),"source":"../source-secret.png"});
    std::fs::write(p.join("document.json"), v.to_string()).unwrap();
    assert!(Editor::open(&p).is_err());
    std::fs::remove_dir_all(p).unwrap();
}
#[test]
fn unsupported_project_version_is_rejected() {
    let d = Document {
        version: 10,
        ..Document::default()
    };
    let p = std::env::temp_dir().join(format!("framewell-test-{}", nonce()));
    std::fs::create_dir(&p).unwrap();
    std::fs::write(
        p.join("document.json"),
        serde_json::json!({"document":d,"source":"source-1.png"}).to_string(),
    )
    .unwrap();
    assert!(Editor::open(&p).is_err());
    std::fs::remove_dir_all(p).unwrap();
}
#[test]
fn guide_html_cannot_execute_caption() {
    let p = std::env::temp_dir().join(format!("framewell-test-{}", nonce()));
    export_guide(&[("<script>alert(1)</script>".into(), sample())], &p).unwrap();
    let html = std::fs::read_to_string(p.join("index.html")).unwrap();
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(p.join("step-1.png").exists());
    assert_eq!(
        std::fs::read(p.join("cover.png")).unwrap(),
        include_bytes!("../assets/framewell-guide-cover.png")
    );
    assert!(html.contains("cover.png"));
    std::fs::remove_dir_all(p).unwrap();
}
#[test]
fn annotation_outside_canvas_does_not_panic() {
    let image = sample();
    let mut o = object(Tool::Redact);
    o.a = [2000., 2000.];
    o.b = [2200., 2200.];
    assert_eq!(render(&image, &[o]), image);
}

#[test]
fn diagonal_arrow_does_not_steal_empty_corner_selection() {
    let mut arrow = object(Tool::Arrow);
    arrow.a = [10., 10.];
    arrow.b = [100., 100.];
    assert!(arrow.hit([55., 55.]));
    assert!(!arrow.hit([12., 95.]));
}

#[test]
fn blur_cannot_spread_redacted_source_pixels() {
    let mut secret = RgbaImage::from_pixel(60, 60, Rgba([255, 255, 255, 255]));
    let clean = secret.clone();
    for y in 10..40 {
        for x in 10..40 {
            secret.put_pixel(x, y, Rgba([255, 0, 0, 255]));
        }
    }
    let mut blur = object(Tool::Blur);
    blur.a = [0., 0.];
    blur.b = [60., 60.];
    let objects = [object(Tool::Redact), blur];
    assert_eq!(render(&secret, &objects), render(&clean, &objects));
}

#[test]
fn maximal_imported_id_does_not_overflow_or_duplicate() {
    let mut e = Editor::new(sample());
    let mut old = object(Tool::Arrow);
    old.id = u64::MAX;
    e.state.document.objects.push(old);
    e.add(object(Tool::Text));
    assert_eq!(e.state.document.objects.len(), 2);
    assert_ne!(
        e.state.document.objects[0].id,
        e.state.document.objects[1].id
    );
}
