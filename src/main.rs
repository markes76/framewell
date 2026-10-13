mod guide;
mod platform;
mod scroll_ui;
mod widgets;
mod workspace;
use eframe::egui::{self, *};
use framewell::*;
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    time::Duration,
};
use widgets::*;
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
    library: Vec<LibraryItem>,
    library_page: usize,
    workspace: Workspace,
    inspector: Inspector,
    project_path: Option<PathBuf>,
    modified: bool,
    pending_open: Option<PendingAction>,
    current_thumb: Option<TextureHandle>,
    thumb_revision: u64,
    guide_textures: Vec<TextureHandle>,
    guide_modified: bool,
    allow_close: bool,
    root: PathBuf,
    search: String,
    steps: Vec<(String, image::RgbaImage)>,
    focus: String,
    proposal: Option<String>,
    job: Option<Receiver<Job>>,
    capture_mode: String,
    zoom: f32,
    style_editing: bool,
    scroll_preview: Option<scroll_ui::ScrollPreview>,
    scroll_run: Option<scroll_ui::ScrollRun>,
}
#[derive(Clone, Copy, PartialEq)]
enum Workspace {
    Editor,
    Library,
    Guide,
}
#[derive(Clone, Copy, PartialEq)]
enum Inspector {
    Properties,
    Layers,
    AI,
}
#[derive(Clone)]
enum PendingAction {
    Open(PathBuf),
    Import,
    Capture,
    Quit,
}
struct LibraryItem {
    path: PathBuf,
    title: String,
    texture: Option<TextureHandle>,
}
enum Job {
    Capture(std::result::Result<Option<PathBuf>, String>),
    AI(u64, std::result::Result<String, String>),
    ScrollPreview(std::result::Result<Option<PathBuf>, String>),
    Scrolling(std::result::Result<Option<image::RgbaImage>, String>),
}
impl Framewell {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_theme(ThemePreference::Light);
        cc.egui_ctx.set_visuals(Visuals::light());
        // Use the installed platform font; Apple font files are never bundled.
        if let Ok(bytes) = std::fs::read("/System/Library/Fonts/SFNS.ttf") {
            let mut fonts = FontDefinitions::default();
            fonts
                .font_data
                .insert("macOS UI".into(), FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "macOS UI".into());
            cc.egui_ctx.set_fonts(fonts);
        }
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = vec2(8., 8.);
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(14.));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(14.));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(12.));
        style.visuals.override_text_color = Some(INK);
        style.visuals.panel_fill = Color32::from_rgb(249, 250, 251);
        style.visuals.widgets.inactive.bg_fill = Color32::WHITE;
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER);
        style.visuals.widgets.hovered.bg_fill = TINT;
        style.visuals.widgets.active.bg_fill = TINT;
        style.spacing.button_padding = vec2(12., 8.);
        style.visuals.selection.bg_fill = Color32::from_rgb(32, 110, 108);
        cc.egui_ctx.set_style(style);
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let root = if let Some(path) = std::env::var_os("FRAMEWELL_LIBRARY_DIR") {
            PathBuf::from(path)
        } else if cfg!(target_os = "macos") {
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
            library_page: 0,
            workspace: Workspace::Editor,
            inspector: Inspector::Properties,
            project_path: None,
            modified: false,
            pending_open: None,
            current_thumb: None,
            thumb_revision: u64::MAX,
            guide_textures: vec![],
            guide_modified: false,
            allow_close: false,
            root,
            search: String::new(),
            steps: vec![],
            focus: "Explain the important controls".into(),
            proposal: None,
            job: None,
            capture_mode: "Region".into(),
            zoom: 1.,
            style_editing: false,
            scroll_preview: None,
            scroll_run: None,
        };
        s.editor.state.document.title = "Example capture (synthetic)".into();
        s.refresh_library();
        match guide::load(&s.root.join("guide")) {
            Ok(steps) => s.steps = steps,
            Err(e) if s.root.join("guide/guide.json").exists() => {
                s.status = format!("Could not restore guide: {e}")
            }
            Err(_) => {}
        }
        s
    }
    fn replace_editor(&mut self, mut editor: Editor) {
        editor.revision = self.editor.revision + 1;
        self.editor = editor;
        self.selected = None;
        self.project_path = None;
        self.zoom = 1.;
        self.changed();
        self.modified = false;
    }
    fn refresh_library(&mut self) {
        self.library_page = 0;
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&self.root)
            .map(|r| {
                r.filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.join("document.json").exists())
                    .collect()
            })
            .unwrap_or_default();
        paths.sort();
        paths.reverse();
        self.library = paths
            .into_iter()
            .filter_map(|path| {
                let bytes = std::fs::read(path.join("document.json")).ok()?;
                if bytes.len() > 8_000_000 {
                    return None;
                }
                let json: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
                let title = json
                    .get("document")
                    .and_then(|d| d.get("title"))
                    .or_else(|| json.get("title"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("Untitled capture")
                    .to_owned();
                Some(LibraryItem {
                    path,
                    title,
                    texture: None,
                })
            })
            .collect();
    }
    fn changed(&mut self) {
        self.dirty = true;
        self.modified = true;
        self.proposal = None;
    }
    fn save(&mut self) {
        let path = self
            .project_path
            .clone()
            .unwrap_or_else(|| self.root.join(format!("capture-{}", nonce())));
        match self.editor.save(&path) {
            Ok(()) => {
                self.project_path = Some(path);
                self.modified = false;
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
        if let Err(e) = platform::ensure_capture_permission() {
            self.status = e.to_string();
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.job = Some(rx);
        self.status = "Choose your capture. Escape cancels.".into();
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        let mode = self.capture_mode.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(400));
            let result = platform::capture(if mode == "Scrolling" {
                "Display"
            } else {
                &mode
            })
            .map_err(|e| e.to_string());
            let _ = tx.send(if mode == "Scrolling" {
                Job::ScrollPreview(result)
            } else {
                Job::Capture(result)
            });
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
                Job::ScrollPreview(result) => match result {
                    Ok(Some(path)) => {
                        let loaded = load_image(&path);
                        let _ = std::fs::remove_file(path);
                        match (loaded, platform::primary_display_bounds()) {
                            (Ok(image), Ok(bounds)) => {
                                let texture = ctx.load_texture(
                                    "scrolling selection",
                                    ColorImage::from_rgba_unmultiplied(
                                        [image.width() as usize, image.height() as usize],
                                        image.as_raw(),
                                    ),
                                    TextureOptions::LINEAR,
                                );
                                self.scroll_preview = Some(scroll_ui::ScrollPreview {
                                    image,
                                    texture,
                                    bounds,
                                    start: None,
                                    area: None,
                                    error: String::new(),
                                });
                            }
                            (Err(e), _) | (_, Err(e)) => self.status = e.to_string(),
                        }
                    }
                    Ok(None) => {
                        self.status = "Scrolling capture cancelled. Your image is unchanged.".into()
                    }
                    Err(e) => self.status = e,
                },
                Job::Scrolling(result) => {
                    self.scroll_run = None;
                    match result {
                        Ok(Some(image)) => {
                            self.replace_editor(Editor::new(image));
                            self.editor.state.document.title = "Scrolling capture".into();
                            self.modified = true;
                            self.workspace = Workspace::Editor;
                            self.status =
                                "Scrolling capture ready. Review the joins before sharing.".into();
                        }
                        Ok(None) => {
                            self.status =
                                "Scrolling capture cancelled. Your image is unchanged.".into()
                        }
                        Err(e) => {
                            self.status = format!(
                                "Scrolling capture stopped: {e} Your previous image is unchanged."
                            )
                        }
                    }
                }
                Job::Capture(Ok(Some(p))) => {
                    match load_image(&p) {
                        Ok(img) => {
                            self.replace_editor(Editor::new(img));
                            self.selected = None;
                            self.changed();
                            self.workspace = Workspace::Editor;
                            self.editor.state.document.title = "New capture".into();
                            self.status = "Captured · Add explanations, then save or export".into();
                        }
                        Err(e) => self.status = e.to_string(),
                    }
                    let _ = std::fs::remove_file(p);
                }
                Job::Capture(Err(e)) => self.status = e,
                Job::Capture(Ok(None)) => {
                    self.status = "Capture cancelled. Your image is unchanged.".into()
                }
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
                self.width.max(24.)
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
        painter.rect_filled(area, 0, CANVAS);
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
        if response.clicked() && self.tool == Tool::Select {
            self.sync_selection();
        }
        if response.drag_started()
            && let Some(p) = drag_press_origin(ui, &response)
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
                    self.sync_selection();
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
                Stroke::new(1.5_f32, Color32::from_rgb(37, 117, 180)),
                StrokeKind::Outside,
            );
        }
    }
}
impl eframe::App for Framewell {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.jobs(ctx);
        ctx.style_mut(|style| {
            style.visuals.override_text_color = Some(INK);
            style.visuals.weak_text_color = Some(MUTED);
            style.visuals.selection = egui::style::Selection {
                bg_fill: TINT,
                stroke: Stroke::new(1.0_f32, ACCENT),
            };
            style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, INK);
            style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, INK);
            style.visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, ACCENT);
        });
        if ctx.input(|i| i.viewport().close_requested())
            && !self.allow_close
            && (self.modified || self.guide_modified)
        {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.pending_open = Some(PendingAction::Quit);
        }
        self.load_thumbnails(ctx);
        if !ctx.wants_keyboard_input()
            && ctx.input(|i| i.modifiers.command && i.key_pressed(Key::Z))
        {
            if ctx.input(|i| i.modifiers.shift) {
                self.editor.redo();
            } else {
                self.editor.undo();
            }
            self.selected = None;
            self.changed();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::S)) {
            if self.workspace == Workspace::Guide {
                self.save_guide();
            } else {
                self.save();
            }
        }
        if self.scroll_preview.is_none() && !ctx.wants_keyboard_input() {
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                self.selected = None;
                self.tool = Tool::Select;
            }
            if ctx.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
                self.delete_selected();
            }
        }
        self.header(ctx);
        if self.workspace == Workspace::Editor {
            self.tool_bar(ctx);
        }
        self.footer(ctx);
        if self.workspace == Workspace::Editor {
            self.filmstrip(ctx);
            self.properties(ctx);
            CentralPanel::default()
                .frame(egui::Frame::NONE.fill(CANVAS))
                .show(ctx, |ui| {
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(246, 248, 250))
                        .inner_margin(Margin::symmetric(18, 9))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let r = ui.add(
                                    TextEdit::singleline(&mut self.editor.state.document.title)
                                        .desired_width(300.)
                                        .frame(false),
                                );
                                if r.changed() {
                                    self.editor.revision += 1;
                                    self.modified = true;
                                }
                                if self.modified {
                                    ui.label(RichText::new("Unsaved").small().color(MUTED));
                                }
                            });
                        });
                    self.canvas(ui);
                });
        } else {
            CentralPanel::default()
                .frame(
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(243, 246, 248))
                        .inner_margin(24),
                )
                .show(ctx, |ui| {
                    if self.workspace == Workspace::Library {
                        self.library_view(ui);
                    } else {
                        self.guide_view(ui);
                    }
                });
        }
        if let Some(action) = self.pending_open.clone() {
            egui::Window::new("Save your changes?")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("You have changes that have not been saved.");
                    ui.horizontal(|ui| {
                        if ui.button("Save and continue").clicked() {
                            if self.modified {
                                self.save();
                            }
                            if self.guide_modified {
                                self.save_guide();
                            }
                            if !self.modified && !self.guide_modified {
                                self.perform_action(action.clone(), ctx);
                                self.pending_open = None;
                            }
                        }
                        if ui.button("Discard and continue").clicked() {
                            self.perform_action(action, ctx);
                            self.pending_open = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.pending_open = None;
                        }
                    });
                });
        }
        self.scrolling_ui(ctx);
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
            .with_min_inner_size([1100., 700.]),
        ..Default::default()
    };
    eframe::run_native(
        "Framewell Rust",
        options,
        Box::new(|cc| Ok(Box::new(Framewell::new(cc)))),
    )
}
