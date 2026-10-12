mod platform;
use eframe::egui::{self, *};
use framewell::*;
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::Duration,
};
struct Framewell {
    editor: Editor,
    tool: Tool,
    selected: Option<usize>,
    draft: Option<Object>,
    drag_origin: Option<Object>,
    color: Color32,
    width: f32,
    text: String,
    texture: Option<TextureHandle>,
    dirty: bool,
    status: String,
    library: Vec<PathBuf>,
    root: PathBuf,
    search: String,
    steps: Vec<(String, image::RgbaImage)>,
    focus: String,
    proposal: Option<String>,
    job: Option<Receiver<Job>>,
    capture_mode: String,
    zoom: f32,
    style_editing: bool,
}
enum Job {
    Capture(std::result::Result<PathBuf, String>),
    AI(u64, std::result::Result<String, String>),
}
impl Framewell {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(Visuals::light());
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = vec2(10., 9.);
        style.spacing.button_padding = vec2(12., 8.);
        style.visuals.selection.bg_fill = Color32::from_rgb(32, 110, 108);
        cc.egui_ctx.set_style(style);
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let root = if cfg!(target_os = "macos") {
            home.join("Library/Application Support/FramewellRust")
        } else {
            home.join(".local/share/framewell")
        };
        let mut s = Self {
            editor: Editor::new(sample()),
            tool: Tool::Select,
            selected: None,
            draft: None,
            drag_origin: None,
            color: Color32::from_rgb(32, 110, 108),
            width: 4.,
            text: "Explain this control".into(),
            texture: None,
            dirty: true,
            status: "Ready · Synthetic demo loaded".into(),
            library: vec![],
            root,
            search: String::new(),
            steps: vec![],
            focus: "Explain the important controls".into(),
            proposal: None,
            job: None,
            capture_mode: "Region".into(),
            zoom: 1.,
            style_editing: false,
        };
        s.refresh_library();
        s
    }
    fn replace_editor(&mut self, mut editor: Editor) {
        editor.revision = self.editor.revision + 1;
        self.editor = editor;
        self.selected = None;
        self.changed();
    }
    fn refresh_library(&mut self) {
        self.library = std::fs::read_dir(&self.root)
            .map(|r| {
                r.filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.join("document.json").exists())
                    .collect()
            })
            .unwrap_or_default();
        self.library.sort();
    }
    fn changed(&mut self) {
        self.dirty = true;
        self.proposal = None;
    }
    fn save(&mut self) {
        let path = self.root.join(format!("capture-{}", nonce()));
        match self.editor.save(&path) {
            Ok(()) => {
                self.status = "Saved to your local library".into();
                self.refresh_library();
            }
            Err(e) => self.status = format!("Save failed: {e}"),
        }
    }
    fn import(&mut self) {
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg"])
            .pick_file()
        {
            match load_image(&p) {
                Ok(img) => {
                    self.replace_editor(Editor::new(img));
                    self.editor.state.document.title =
                        p.file_stem().unwrap_or_default().to_string_lossy().into();
                    self.selected = None;
                    self.changed();
                    self.status = "Image imported locally".into();
                }
                Err(e) => self.status = e.to_string(),
            }
        }
    }
    fn start_capture(&mut self, ctx: &Context) {
        if self.job.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.job = Some(rx);
        self.status = "Choose your capture. Escape cancels.".into();
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        let mode = self.capture_mode.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(400));
            let _ = tx.send(Job::Capture(
                platform::capture(&mode).map_err(|e| e.to_string()),
            ));
        });
    }
    fn start_ai(&mut self) {
        if self.job.is_some() {
            return;
        }
        let path = std::env::temp_dir().join(format!("framewell-ai-image-{}.png", nonce()));
        if let Err(e) = self.editor.render().save(&path) {
            self.status = e.to_string();
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.job = Some(rx);
        self.status = "Drafting an explanation using your Codex account…".into();
        let revision = self.editor.revision;
        let focus = self.focus.clone();
        std::thread::spawn(move || {
            let _ = tx.send(Job::AI(
                revision,
                platform::explain(path, focus).map_err(|e| e.to_string()),
            ));
        });
    }
    fn jobs(&mut self, ctx: &Context) {
        let result = self.job.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(result) = result {
            self.job = None;
            ctx.send_viewport_cmd(ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
            match result {
                Job::Capture(Ok(p)) => {
                    match load_image(&p) {
                        Ok(img) => {
                            self.replace_editor(Editor::new(img));
                            self.selected = None;
                            self.changed();
                            self.status = "Captured · Add explanations, then save or export".into();
                        }
                        Err(e) => self.status = e.to_string(),
                    }
                    let _ = std::fs::remove_file(p);
                }
                Job::Capture(Err(e)) => self.status = e,
                Job::AI(revision, result) => {
                    if revision != self.editor.revision {
                        self.status =
                            "Image changed during the AI request. Draft discarded.".into();
                    } else {
                        match result {
                            Ok(s) => {
                                self.proposal = Some(s);
                                self.status = "AI draft ready. Review before applying.".into();
                            }
                            Err(e) => self.status = e,
                        }
                    }
                }
            }
        }
        if self.job.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
    fn new_object(&self, a: [f32; 2]) -> Object {
        let text = if self.tool == Tool::Step {
            (self
                .editor
                .state
                .document
                .objects
                .iter()
                .filter(|o| o.tool == Tool::Step)
                .count()
                + 1)
            .to_string()
        } else {
            self.text.clone()
        };
        Object {
            id: 0,
            tool: self.tool,
            a,
            b: a,
            color: self.color.to_array(),
            width: if matches!(self.tool, Tool::Text | Tool::Callout) {
                24.
            } else {
                self.width
            },
            text,
            points: vec![a],
        }
    }
    fn canvas(&mut self, ui: &mut Ui) {
        let mut objects = self.editor.state.document.objects.clone();
        if let Some(o) = &self.draft {
            objects.push(o.clone());
        }
        if self.dirty || self.texture.is_none() {
            let img = render(&self.editor.state.pixels, &objects);
            let size = [img.width() as usize, img.height() as usize];
            let color = ColorImage::from_rgba_unmultiplied(size, img.as_raw());
            if let Some(t) = &mut self.texture {
                t.set(color, TextureOptions::LINEAR)
            } else {
                self.texture = Some(
                    ui.ctx()
                        .load_texture("canvas", color, TextureOptions::LINEAR),
                );
            }
            self.dirty = false;
        }
        let available = ui.available_size();
        let w = self.editor.state.pixels.width() as f32;
        let h = self.editor.state.pixels.height() as f32;
        let scale = ((available.x - 48.) / w)
            .min((available.y - 48.) / h)
            .max(0.02)
            * self.zoom;
        let size = vec2(w, h) * scale;
        let (area, _) = ui.allocate_exact_size(available, Sense::hover());
        let rect = Rect::from_center_size(area.center(), size);
        let response = ui.interact(rect, ui.id().with("canvas"), Sense::click_and_drag());
        let painter = ui.painter_at(area);
        painter.rect_filled(area, 0, Color32::from_rgb(233, 238, 239));
        painter.rect_filled(rect.expand(2.), 2, Color32::WHITE);
        painter.image(
            self.texture.as_ref().unwrap().id(),
            rect,
            Rect::from_min_max(pos2(0., 0.), pos2(1., 1.)),
            Color32::WHITE,
        );
        let point = |p: Pos2| {
            let p = (p - rect.min) / scale;
            [p.x.clamp(0., w), p.y.clamp(0., h)]
        };
        if response.clicked()
            && self.tool == Tool::Select
            && let Some(p) = response.interact_pointer_pos()
        {
            let p = point(p);
            self.selected = self
                .editor
                .state
                .document
                .objects
                .iter()
                .rposition(|o| o.hit(p));
        }
        if response.drag_started()
            && let Some(p) = response.interact_pointer_pos()
        {
            let p = point(p);
            if self.tool == Tool::Select {
                self.selected = self
                    .editor
                    .state
                    .document
                    .objects
                    .iter()
                    .rposition(|o| o.hit(p));
                if let Some(i) = self.selected {
                    self.editor.checkpoint();
                    self.drag_origin = Some(self.editor.state.document.objects[i].clone());
                }
            } else {
                self.draft = Some(self.new_object(p));
            }
        }
        if response.dragged() {
            if self.tool == Tool::Select {
                if let (Some(origin), Some(i)) = (&self.drag_origin, self.selected) {
                    let delta = response.drag_delta() / scale;
                    let o = &mut self.editor.state.document.objects[i];
                    o.a = [origin.a[0] + delta.x, origin.a[1] + delta.y];
                    o.b = [origin.b[0] + delta.x, origin.b[1] + delta.y];
                    o.points = origin
                        .points
                        .iter()
                        .map(|p| [p[0] + delta.x, p[1] + delta.y])
                        .collect();
                    self.dirty = true;
                }
            } else if let (Some(o), Some(p)) = (&mut self.draft, response.interact_pointer_pos()) {
                o.b = point(p);
                if o.tool == Tool::Pen {
                    o.points.push(o.b);
                }
                self.dirty = true;
            }
        }
        if response.drag_stopped() {
            self.drag_origin = None;
            if let Some(mut o) = self.draft.take() {
                if matches!(o.tool, Tool::Text | Tool::Callout) && o.bounds()[2] < 50. {
                    o.b = [o.a[0] + 250., o.a[1] + 60.];
                }
                if o.tool == Tool::Step {
                    o.b = [o.a[0] + 40., o.a[1] + 40.];
                }
                self.editor.add(o);
                self.selected = Some(self.editor.state.document.objects.len() - 1);
            }
            self.changed();
        }
        if let Some(i) = self.selected
            && let Some(o) = self.editor.state.document.objects.get(i)
        {
            let [x, y, w, h] = o.bounds();
            let r = Rect::from_min_size(rect.min + vec2(x, y) * scale, vec2(w, h) * scale);
            painter.rect_stroke(
                r.expand(5.),
                2,
                Stroke::new(1.5, Color32::from_rgb(37, 117, 180)),
                StrokeKind::Outside,
            );
        }
    }
}
impl eframe::App for Framewell {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.jobs(ctx);
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::Z)) {
            self.editor.undo();
            self.selected = None;
            self.changed();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::S)) {
            self.save();
        }
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(7.);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Framewell").color(Color32::from_rgb(29, 55, 65)));
                ui.label(
                    RichText::new("RUST / EARLY BUILD")
                        .small()
                        .color(Color32::GRAY),
                );
                ui.separator();
                ComboBox::from_id_salt("capture")
                    .selected_text(&self.capture_mode)
                    .show_ui(ui, |ui| {
                        for mode in ["Region", "Window", "Display"] {
                            ui.selectable_value(&mut self.capture_mode, mode.into(), mode);
                        }
                    });
                if ui
                    .add_enabled(
                        self.job.is_none(),
                        Button::new("Capture").fill(Color32::from_rgb(205, 234, 229)),
                    )
                    .clicked()
                {
                    self.start_capture(ctx);
                }
                if ui.button("Import image").clicked() {
                    self.import();
                }
                if ui.button("Save project").clicked() {
                    self.save();
                }
                if ui.button("Export PNG").clicked()
                    && let Some(p) = rfd::FileDialog::new()
                        .add_filter("PNG", &["png"])
                        .set_file_name("framewell.png")
                        .save_file()
                {
                    match self.editor.render().save(p) {
                        Ok(()) => self.status = "PNG exported with opaque redactions".into(),
                        Err(e) => self.status = e.to_string(),
                    }
                }
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                for tool in Tool::ALL {
                    if ui
                        .selectable_label(self.tool == tool, format!("{tool:?}"))
                        .clicked()
                    {
                        self.tool = tool;
                    }
                }
                ui.separator();
                if ui.button("Undo").clicked() {
                    self.editor.undo();
                    self.selected = None;
                    self.changed();
                }
                if ui.button("Redo").clicked() {
                    self.editor.redo();
                    self.selected = None;
                    self.changed();
                }
            });
            ui.add_space(5.);
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(format!(
                        "{} × {}",
                        self.editor.state.pixels.width(),
                        self.editor.state.pixels.height()
                    ));
                    ui.add(Slider::new(&mut self.zoom, 0.5..=2.).text("Zoom"));
                });
            });
        });
        egui::SidePanel::left("library")
            .default_width(180.)
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_space(14.);
                ui.heading("Library");
                ui.label(
                    RichText::new("Saved on this computer")
                        .small()
                        .color(Color32::GRAY),
                );
                ui.text_edit_singleline(&mut self.search);
                ui.add_space(12.);
                if ui.button("Synthetic demo").clicked() {
                    self.replace_editor(Editor::new(sample()));
                    self.selected = None;
                    self.changed();
                }
                ScrollArea::vertical().show(ui, |ui| {
                    for p in self.library.clone() {
                        let name = p.file_name().unwrap_or_default().to_string_lossy();
                        if !name.to_lowercase().contains(&self.search.to_lowercase()) {
                            continue;
                        }
                        if ui.button(name.as_ref()).clicked() {
                            match Editor::open(&p) {
                                Ok(e) => {
                                    self.replace_editor(e);
                                    self.selected = None;
                                    self.changed();
                                    self.status = "Saved project opened".into();
                                }
                                Err(e) => self.status = e.to_string(),
                            }
                        }
                    }
                });
                ui.separator();
                ui.heading("Guide");
                ui.label(format!("{} steps", self.steps.len()));
                if ui.button("Add current image").clicked() {
                    self.steps.push((
                        self.editor.state.document.caption.clone(),
                        self.editor.render(),
                    ));
                    self.status = "Added to guide".into();
                }
                if ui
                    .add_enabled(!self.steps.is_empty(), Button::new("Export HTML guide"))
                    .clicked()
                    && let Some(p) = rfd::FileDialog::new().pick_folder()
                {
                    match export_guide(&self.steps, &p.join(format!("framewell-guide-{}", nonce())))
                    {
                        Ok(()) => self.status = "Guide exported with images".into(),
                        Err(e) => self.status = e.to_string(),
                    }
                }
                for (i, (caption, _)) in self.steps.iter_mut().enumerate() {
                    ui.label(format!("Step {}", i + 1));
                    ui.text_edit_multiline(caption);
                }
            });
        egui::SidePanel::right("properties").default_width(260.).resizable(true).show(ctx, |ui| {
            ui.add_space(14.); ui.heading("Style");
            let mut style_changed = false;
            ui.horizontal(|ui| { ui.label("Color"); style_changed |= ui.color_edit_button_srgba(&mut self.color).changed(); });
            style_changed |= ui.add(Slider::new(&mut self.width, 1.0..=36.).text("Stroke / text size")).changed();
            ui.label("Text / callout"); style_changed |= ui.text_edit_multiline(&mut self.text).changed();
            if style_changed && let Some(i) = self.selected {
                if !self.style_editing { self.editor.checkpoint(); self.style_editing = true; }
                let item = &mut self.editor.state.document.objects[i];
                item.color = self.color.to_array(); item.width = self.width;
                if matches!(item.tool, Tool::Text | Tool::Callout) { item.text = self.text.clone(); }
                self.changed();
            }
            if !ctx.input(|i| i.pointer.any_down()) && !style_changed { self.style_editing = false; }
            if let Some(i) = self.selected && let Some(o) = self.editor.state.document.objects.get(i) {
                ui.separator(); ui.label(format!("Selected {:?} · #{}", o.tool, o.id));
                if ui.button("Delete layer").clicked() { self.editor.checkpoint(); self.editor.state.document.objects.remove(i); self.selected = None; self.changed(); }
            }
            ui.separator(); ui.heading("Layers");
            for (i, o) in self.editor.state.document.objects.iter().enumerate().rev() {
                if ui.selectable_label(self.selected == Some(i), format!("{:?} #{}", o.tool, o.id)).clicked() {
                    self.selected = Some(i); self.text = o.text.clone(); self.color = Color32::from_rgba_unmultiplied(o.color[0], o.color[1], o.color[2], o.color[3]); self.width = o.width;
                }
            }
            ui.separator(); ui.heading("AI explanation");
            ui.label(RichText::new("Uses your Codex login. Sends the current rendered image when requested. Account limits apply.").small());
            ui.text_edit_multiline(&mut self.focus);
            if ui.add_enabled(self.job.is_none(), Button::new("Draft explanation")).clicked() { self.start_ai(); }
            if self.job.is_some() { ui.spinner(); }
            if let Some(proposal) = &mut self.proposal {
                ui.text_edit_multiline(proposal);
                if ui.button("Use as guide caption").clicked() { let p = proposal.clone(); self.editor.checkpoint(); self.editor.state.document.caption = p; self.proposal = None; self.status = "Caption applied. Undo restores the previous caption.".into(); }
            }
            ui.label("Guide caption");
            if ui.text_edit_multiline(&mut self.editor.state.document.caption).changed() { self.editor.revision += 1; self.proposal = None; }
        });
        CentralPanel::default().show(ctx, |ui| self.canvas(ui));
    }
}
fn main() -> eframe::Result {
    if std::env::args().any(|a| a == "--self-check") {
        let e = Editor::new(sample());
        let p = std::env::temp_dir().join("framewell-rust-self-check");
        e.save(&p).unwrap();
        let restored = Editor::open(&p).unwrap();
        assert_eq!(restored.state.pixels.dimensions(), (1080, 680));
        e.render().save(p.join("demo.png")).unwrap();
        println!("Self-check passed: document save/open and PNG render");
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1380., 850.])
            .with_min_inner_size([1000., 640.]),
        ..Default::default()
    };
    eframe::run_native(
        "Framewell Rust",
        options,
        Box::new(|cc| Ok(Box::new(Framewell::new(cc)))),
    )
}
