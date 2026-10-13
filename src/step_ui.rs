use super::*;
use framewell::step::{StepPreset, StepSequence, StepShape};
pub(crate) fn marker_preview(
    p: &Painter,
    center: Pos2,
    size: f32,
    preset: &StepPreset,
    text: &str,
) {
    let fill = Color32::from_rgba_unmultiplied(
        preset.fill[0],
        preset.fill[1],
        preset.fill[2],
        preset.fill[3],
    );
    let c = preset.style.border_color;
    let stroke = Stroke::new(
        preset.style.border_width * size / preset.size,
        Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]),
    );
    let r = Rect::from_center_size(center, Vec2::splat(size));
    match preset.style.shape {
        StepShape::Circle => {
            p.circle(center, size / 2., fill, stroke);
        }
        StepShape::Square => {
            p.rect(r, 0, fill, stroke, StrokeKind::Inside);
        }
        StepShape::Diamond => {
            p.add(egui::Shape::convex_polygon(
                vec![
                    r.center_top(),
                    r.right_center(),
                    r.center_bottom(),
                    r.left_center(),
                ],
                fill,
                stroke,
            ));
        }
    }
    let c = preset.style.text_color;
    let font = (preset.font_size * size / preset.size).min(
        size * if preset.style.shape == StepShape::Diamond {
            0.46
        } else {
            0.64
        },
    );
    p.text(
        center,
        Align2::CENTER_CENTER,
        text,
        FontId::new(
            font / (text.len() as f32 / 2.).max(1.),
            FontFamily::Name("Step bold".into()),
        ),
        Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]),
    );
}
fn step_section(ui: &mut Ui, title: &str) {
    ui.add_space(8.);
    ui.label(RichText::new(title).size(13.).strong().color(INK));
}
impl Framewell {
    pub(crate) fn refresh_step_sequence(&mut self) {
        self.step_next = self
            .editor
            .state
            .document
            .objects
            .iter()
            .filter(|o| o.tool == Tool::Step)
            .filter_map(|o| self.step_sequence.value(&o.text))
            .max()
            .unwrap_or(0)
            .saturating_add(1)
            .min(9999);
    }

    pub(crate) fn load_step_presets(&mut self) {
        let path = self.root.join("step-styles.json");
        if std::fs::metadata(&path).is_ok_and(|m| m.len() < 100_000)
            && let Ok(bytes) = std::fs::read(path)
            && let Ok(presets) = serde_json::from_slice::<Vec<StepPreset>>(&bytes)
        {
            self.saved_step_presets = presets
                .into_iter()
                .filter(StepPreset::valid)
                .take(12)
                .collect();
        }
    }
    fn apply_step_style(&mut self) {
        if let Some(i) = self.selected
            && self
                .editor
                .state
                .document
                .objects
                .get(i)
                .is_some_and(|o| o.tool == Tool::Step)
        {
            if !self.style_editing {
                self.editor.checkpoint();
                self.style_editing = true;
            }
            let o = &mut self.editor.state.document.objects[i];
            let [x, y, w, h] = o.bounds();
            let mut replacement = self
                .step_preset
                .marker([x + w / 2., y + h / 2.], o.text.clone());
            replacement.id = o.id;
            *o = replacement;
            self.changed();
        }
    }
    pub(crate) fn step_properties(&mut self, ui: &mut Ui, ctx: &Context) {
        let editing = self.selected.is_some();
        ui.spacing_mut().item_spacing.y = 5.;
        ui.spacing_mut().button_padding = vec2(8., 5.);
        ui.add_space(10.);
        ui.heading(if editing { "Edit step" } else { "Step markers" });
        ui.label(
            RichText::new(if editing {
                "Style this marker, or return to placing steps."
            } else {
                "One click. One clear instruction."
            })
            .color(MUTED),
        );
        ui.add_space(4.);
        let label = if editing {
            self.editor.state.document.objects[self.selected.unwrap()]
                .text
                .clone()
        } else {
            self.step_sequence.label(self.step_next)
        };
        if editing {
            step_section(ui, "Label");
            let mut value = label;
            if ui
                .add(
                    TextEdit::singleline(&mut value)
                        .char_limit(8)
                        .desired_width(f32::INFINITY),
                )
                .changed()
            {
                self.editor.checkpoint();
                self.editor.state.document.objects[self.selected.unwrap()].text = value;
                self.changed();
            }
            if ui.button("Continue placing steps").clicked() {
                self.selected = None;
                self.tool = Tool::Step;
            }
        } else {
            step_section(ui, "Sequence");
            ui.horizontal(|ui| {
                for (value, label) in [
                    (StepSequence::Numbers, "1, 2, 3"),
                    (StepSequence::Uppercase, "A, B, C"),
                    (StepSequence::Lowercase, "a, b, c"),
                ] {
                    if ui
                        .selectable_label(self.step_sequence == value, label)
                        .clicked()
                    {
                        self.step_sequence = value;
                        self.step_next = 1;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Next value");
                ui.add(DragValue::new(&mut self.step_next).range(1..=9999));
                if ui.button("Restart").clicked() {
                    self.step_next = 1;
                }
            });
        }
        step_section(ui, "Quick styles");
        let fills = [
            [23, 111, 118, 255],
            [207, 82, 61, 255],
            [51, 91, 186, 255],
            [39, 49, 65, 255],
        ];
        let names = ["Teal", "Coral", "Cobalt", "Ink"];
        ui.horizontal(|ui| {
            for i in 0..4 {
                let mut preset = StepPreset {
                    name: names[i].into(),
                    fill: fills[i],
                    ..StepPreset::default()
                };
                preset.style.shape = if i == 2 {
                    StepShape::Square
                } else if i == 3 {
                    StepShape::Diamond
                } else {
                    StepShape::Circle
                };
                let (r, response) = ui.allocate_exact_size(vec2(56., 54.), Sense::click());
                if response.hovered() {
                    ui.painter().rect_filled(r, 6, TINT);
                }
                marker_preview(
                    ui.painter(),
                    r.center_top() + vec2(0., 20.),
                    32.,
                    &preset,
                    "1",
                );
                ui.painter().text(
                    r.center_bottom() - vec2(0., 7.),
                    Align2::CENTER_CENTER,
                    names[i],
                    FontId::proportional(11.),
                    INK,
                );
                if response.on_hover_text(names[i]).clicked() {
                    self.step_preset = preset;
                    self.apply_step_style();
                    self.style_editing = false;
                }
            }
        });
        step_section(ui, "Shape & size");
        let mut changed = false;
        ui.horizontal(|ui| {
            for (value, label) in [
                (StepShape::Circle, "Circle"),
                (StepShape::Square, "Square"),
                (StepShape::Diamond, "Diamond"),
            ] {
                changed |= ui
                    .selectable_value(&mut self.step_preset.style.shape, value, label)
                    .changed();
            }
        });
        changed |= ui
            .add(
                Slider::new(&mut self.step_preset.size, 24.0..=160.)
                    .text("Size")
                    .suffix(" px"),
            )
            .changed();
        changed |= ui
            .add(
                Slider::new(&mut self.step_preset.font_size, 10.0..=80.)
                    .text("Text")
                    .suffix(" px"),
            )
            .changed();
        ui.horizontal(|ui| {
            ui.label("Fill");
            changed |= ui
                .color_edit_button_srgba_unmultiplied(&mut self.step_preset.fill)
                .changed();
            ui.label("Text");
            changed |= ui
                .color_edit_button_srgba_unmultiplied(&mut self.step_preset.style.text_color)
                .changed();
            ui.label("Border");
            changed |= ui
                .color_edit_button_srgba_unmultiplied(&mut self.step_preset.style.border_color)
                .changed();
        });
        changed |= ui
            .add(
                Slider::new(&mut self.step_preset.style.border_width, 0.0..=12.)
                    .text("Border")
                    .suffix(" px"),
            )
            .changed();
        if changed {
            self.apply_step_style();
        }
        if !ctx.input(|i| i.pointer.any_down()) && !changed {
            self.style_editing = false;
        }
        if ui.button("Apply style to all steps").clicked() {
            self.editor.checkpoint();
            for o in &mut self.editor.state.document.objects {
                if o.tool == Tool::Step {
                    let [x, y, w, h] = o.bounds();
                    let mut next = self
                        .step_preset
                        .marker([x + w / 2., y + h / 2.], o.text.clone());
                    next.id = o.id;
                    *o = next;
                }
            }
            self.changed();
        }
        egui::CollapsingHeader::new("My saved styles").show(ui, |ui| {
            for preset in self.saved_step_presets.clone() {
                if ui.button(&preset.name).clicked() {
                    self.step_preset = preset;
                    self.apply_step_style();
                    self.style_editing = false;
                }
            }
            ui.add(
                TextEdit::singleline(&mut self.step_preset_name)
                    .char_limit(40)
                    .hint_text("Style name"),
            );
            if ui.button("Save current style").clicked() {
                let mut preset = self.step_preset.clone();
                preset.name = self.step_preset_name.trim().into();
                if !preset.name.is_empty() {
                    self.saved_step_presets.retain(|p| p.name != preset.name);
                    if self.saved_step_presets.len() == 12 {
                        self.saved_step_presets.remove(0);
                    }
                    self.saved_step_presets.push(preset);
                    let result = (|| -> framewell::Result<()> {
                        std::fs::create_dir_all(&self.root)?;
                        std::fs::write(
                            self.root.join("step-styles.json"),
                            serde_json::to_vec_pretty(&self.saved_step_presets)?,
                        )?;
                        Ok(())
                    })();
                    self.status = match result {
                        Ok(()) => "Step style saved for your next capture".into(),
                        Err(e) => format!("Could not save style: {e}"),
                    };
                }
            }
        });
        if editing && ui.button("Delete this step").clicked() {
            self.delete_selected();
        }
        ui.add_space(12.);
        ui.label(
            RichText::new("Use Move to select a marker. Its label, shape and style stay editable.")
                .small()
                .color(MUTED),
        );
    }
}
