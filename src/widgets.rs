//! Original, consistent vector controls for the desktop workspace.
use eframe::egui::*;
use framewell::Tool;

pub const INK: Color32 = Color32::from_rgb(37, 44, 52);
pub const MUTED: Color32 = Color32::from_rgb(85, 96, 107);
pub const ACCENT: Color32 = Color32::from_rgb(27, 105, 111);
pub const TINT: Color32 = Color32::from_rgb(224, 240, 240);
pub const BORDER: Color32 = Color32::from_rgb(218, 224, 229);
pub const CANVAS: Color32 = Color32::from_rgb(231, 235, 239);
pub const COLORS: [Color32; 8] = [
    ACCENT,
    Color32::from_rgb(34, 105, 189),
    Color32::from_rgb(199, 59, 66),
    Color32::from_rgb(155, 82, 184),
    Color32::from_rgb(209, 131, 26),
    Color32::from_rgb(60, 135, 79),
    INK,
    Color32::from_rgb(147, 90, 57),
];
pub fn label(tool: Tool) -> &'static str {
    match tool {
        Tool::Select => "Move",
        Tool::Arrow => "Arrow",
        Tool::Rectangle => "Shape",
        Tool::Highlight => "Highlight",
        Tool::Redact => "Redact",
        Tool::Text => "Text",
        Tool::Callout => "Callout",
        Tool::Step => "Step",
        Tool::Pen => "Pen",
        Tool::Blur => "Blur",
    }
}
pub fn hint(tool: Tool) -> &'static str {
    match tool {
        Tool::Select => "Click an annotation to select it. Drag to move.",
        Tool::Arrow => "Drag from the start of your arrow to its tip.",
        Tool::Rectangle => "Drag to outline an area of the image.",
        Tool::Highlight => "Drag to highlight an area with transparent color.",
        Tool::Redact => "Drag over private information. PNG exports use an opaque mask.",
        Tool::Text => "Enter text in Tool Properties, then drag to place it.",
        Tool::Callout => "Enter an explanation in Tool Properties, then drag to place it.",
        Tool::Step => "Drag to place the next numbered step.",
        Tool::Pen => "Drag to draw freely on the image.",
        Tool::Blur => "Drag to blur an area. Use Redact to protect sensitive information.",
    }
}
pub fn icon(p: &Painter, r: Rect, tool: Tool, color: Color32, width: f32) {
    let r = r.shrink(3.);
    let a = r.left_top();
    let b = r.right_bottom();
    let c = r.center();
    let s = Stroke::new(width, color);
    match tool {
        Tool::Select => {
            p.arrow(c, vec2(0., -r.height() / 2.), s);
            p.arrow(c, vec2(0., r.height() / 2.), s);
            p.arrow(c, vec2(-r.width() / 2., 0.), s);
            p.arrow(c, vec2(r.width() / 2., 0.), s);
        }
        Tool::Arrow => p.arrow(pos2(a.x, b.y), vec2(r.width(), -r.height()), s),
        Tool::Rectangle => {
            p.rect_stroke(r, 2, s, StrokeKind::Inside);
        }
        Tool::Highlight => {
            p.line_segment(
                [pos2(a.x, b.y - 3.), pos2(b.x, b.y - 3.)],
                Stroke::new(7.0_f32, color.gamma_multiply(0.35)),
            );
            p.line_segment(
                [pos2(a.x + 3., b.y - 7.), pos2(b.x - 3., a.y + 2.)],
                Stroke::new(5.0_f32, color),
            );
        }
        Tool::Redact => {
            p.rect_filled(r, 2, color);
        }
        Tool::Text => {
            p.line_segment([a, pos2(b.x, a.y)], s);
            p.line_segment([pos2(c.x, a.y), pos2(c.x, b.y)], s);
            p.line_segment([pos2(c.x - 5., b.y), pos2(c.x + 5., b.y)], s);
        }
        Tool::Callout => {
            p.rect_stroke(
                Rect::from_min_max(a, pos2(b.x, b.y - 6.)),
                3,
                s,
                StrokeKind::Inside,
            );
            p.line_segment([pos2(a.x + 6., b.y - 6.), pos2(a.x + 6., b.y)], s);
            p.line_segment([pos2(a.x + 6., b.y), pos2(a.x + 13., b.y - 6.)], s);
        }
        Tool::Step => {
            p.circle_stroke(c, r.width() / 2., s);
            p.text(
                c,
                Align2::CENTER_CENTER,
                "1",
                FontId::proportional(r.height() * 0.65),
                color,
            );
        }
        Tool::Pen => {
            p.line_segment([pos2(a.x, b.y), pos2(a.x + 7., b.y - 2.)], s);
            p.line_segment([pos2(a.x + 7., b.y - 2.), pos2(b.x, a.y + 3.)], s);
            p.line_segment([pos2(b.x, a.y + 3.), pos2(b.x - 4., a.y)], s);
            p.line_segment([pos2(b.x - 4., a.y), pos2(a.x + 2., b.y - 7.)], s);
        }
        Tool::Blur => {
            for y in 0..4 {
                for x in 0..4 {
                    p.circle_filled(
                        a + vec2(x as f32 * r.width() / 3., y as f32 * r.height() / 3.),
                        1.8,
                        color.gamma_multiply(0.5 + (x + y) as f32 * 0.08),
                    );
                }
            }
        }
    }
}
pub fn tool_button(ui: &mut Ui, tool: Tool, active: bool) -> Response {
    let (r, response) = ui.allocate_exact_size(vec2(67., 66.), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), label(tool)));
    let color = if active { ACCENT } else { INK };
    if active || response.hovered() || response.has_focus() {
        ui.painter().rect_filled(
            r,
            6,
            if active {
                TINT
            } else {
                Color32::from_gray(239)
            },
        );
    }
    if response.has_focus() {
        ui.painter()
            .rect_stroke(r, 6, Stroke::new(1.0_f32, ACCENT), StrokeKind::Inside);
    }
    icon(
        ui.painter(),
        Rect::from_center_size(pos2(r.center().x, r.top() + 23.), vec2(28., 28.)),
        tool,
        color,
        1.8,
    );
    ui.painter().text(
        pos2(r.center().x, r.bottom() - 14.),
        Align2::CENTER_CENTER,
        label(tool),
        FontId::proportional(13.),
        color,
    );
    response.on_hover_text(hint(tool))
}
pub fn style_button(
    ui: &mut Ui,
    tool: Tool,
    color: Color32,
    active: bool,
    variant: usize,
) -> Response {
    let (r, response) = ui.allocate_exact_size(vec2(59., 50.), Sense::click());
    response.widget_info(|| {
        WidgetInfo::labeled(
            WidgetType::Button,
            ui.is_enabled(),
            format!("{} style {}", label(tool), variant + 1),
        )
    });
    ui.painter()
        .rect_filled(r, 5, if active { TINT } else { Color32::WHITE });
    ui.painter().rect_stroke(
        r,
        5,
        Stroke::new(
            if active { 2.0_f32 } else { 1.0_f32 },
            if active { ACCENT } else { BORDER },
        ),
        StrokeKind::Inside,
    );
    icon(
        ui.painter(),
        r.shrink2(vec2(12., 8.)),
        tool,
        color,
        if variant >= 4 { 3. } else { 2. },
    );
    response
}
pub fn image_tile(
    ui: &mut Ui,
    texture: &TextureHandle,
    title: &str,
    size: Vec2,
    active: bool,
) -> Response {
    let (r, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| {
        WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), format!("Open {title}"))
    });
    ui.painter()
        .rect_filled(r, 5, if active { TINT } else { Color32::WHITE });
    ui.painter().rect_stroke(
        r,
        5,
        Stroke::new(
            if active { 2.0_f32 } else { 1.0_f32 },
            if active { ACCENT } else { BORDER },
        ),
        StrokeKind::Inside,
    );
    let image_box =
        Rect::from_min_max(r.min + vec2(7., 7.), pos2(r.right() - 7., r.bottom() - 30.));
    let dimensions = texture.size_vec2();
    let fitted =
        dimensions * (image_box.width() / dimensions.x).min(image_box.height() / dimensions.y);
    ui.painter().image(
        texture.id(),
        Rect::from_center_size(image_box.center(), fitted),
        Rect::from_min_max(Pos2::ZERO, pos2(1., 1.)),
        Color32::WHITE,
    );
    let text = ui.painter().layout(
        title.to_owned(),
        FontId::proportional(12.),
        INK,
        f32::INFINITY,
    );
    ui.painter()
        .with_clip_rect(Rect::from_min_max(
            pos2(r.left() + 7., r.bottom() - 25.),
            r.max - vec2(7., 5.),
        ))
        .galley(pos2(r.left() + 7., r.bottom() - 25.), text, INK);
    response.on_hover_text(title)
}
pub fn section(ui: &mut Ui, title: &str) {
    ui.add_space(14.);
    ui.label(RichText::new(title).size(15.).strong().color(INK));
    ui.add_space(5.);
}
pub fn drag_press_origin(ui: &Ui, response: &Response) -> Option<Pos2> {
    ui.input(|i| i.pointer.press_origin())
        .or_else(|| response.interact_pointer_pos())
}
#[cfg(test)]
mod drag_tests {
    use super::*;
    #[test]
    fn delayed_drag_frame_keeps_the_initial_press_point() {
        let ctx = Context::default();
        let screen = Some(Rect::from_min_size(Pos2::ZERO, vec2(600., 500.)));
        let _ = ctx.run(
            RawInput {
                screen_rect: screen,
                ..Default::default()
            },
            |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    ui.allocate_exact_size(vec2(500., 400.), Sense::click_and_drag());
                });
            },
        );
        let mut start = None;
        let _ = ctx.run(
            RawInput {
                screen_rect: screen,
                events: vec![
                    Event::PointerMoved(pos2(100., 100.)),
                    Event::PointerButton {
                        pos: pos2(100., 100.),
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: Modifiers::NONE,
                    },
                    Event::PointerMoved(pos2(300., 200.)),
                ],
                ..Default::default()
            },
            |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    let (_, r) = ui.allocate_exact_size(vec2(500., 400.), Sense::click_and_drag());
                    start = drag_press_origin(ui, &r);
                });
            },
        );
        assert_eq!(start, Some(pos2(100., 100.)));
    }
}
