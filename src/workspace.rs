use super::*;

impl Framewell {
    pub(crate) fn sync_selection(&mut self) {
        if let Some(o) = self
            .selected
            .and_then(|i| self.editor.state.document.objects.get(i))
        {
            self.text = o.text.clone();
            self.width = o.width;
            self.color =
                Color32::from_rgba_unmultiplied(o.color[0], o.color[1], o.color[2], o.color[3]);
        }
    }
    fn apply_style(&mut self) {
        if let Some(i) = self.selected {
            if !self.style_editing {
                self.editor.checkpoint();
                self.style_editing = true;
            }
            if let Some(o) = self.editor.state.document.objects.get_mut(i) {
                o.color = self.color.to_array();
                o.width = self.width;
                if matches!(o.tool, Tool::Text | Tool::Callout | Tool::Step) {
                    o.text = self.text.clone();
                }
            }
            self.changed();
        }
    }
    pub(crate) fn delete_selected(&mut self) {
        if let Some(i) = self.selected.take()
            && i < self.editor.state.document.objects.len()
        {
            self.editor.checkpoint();
            self.editor.state.document.objects.remove(i);
            self.changed();
        }
    }
    fn request_open(&mut self, path: PathBuf) {
        if self.modified {
            self.pending_open = Some(PendingAction::Open(path));
        } else {
            self.open_project(path);
        }
    }
    fn request_action(&mut self, action: PendingAction, ctx: &Context) {
        if self.modified {
            self.pending_open = Some(action);
        } else {
            self.perform_action(action, ctx);
        }
    }
    pub(crate) fn perform_action(&mut self, action: PendingAction, ctx: &Context) {
        match action {
            PendingAction::Open(path) => self.open_project(path),
            PendingAction::Import => {
                self.import();
                self.workspace = Workspace::Editor;
            }
            PendingAction::Capture => self.start_capture(ctx),
            PendingAction::Quit => {
                self.allow_close = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        }
    }
    pub(crate) fn open_project(&mut self, path: PathBuf) {
        match Editor::open(&path) {
            Ok(e) => {
                self.replace_editor(e);
                self.project_path = Some(path);
                self.workspace = Workspace::Editor;
                self.status = "Project opened · Annotations remain editable".into();
            }
            Err(e) => self.status = format!("Could not open project: {e}"),
        }
    }
    fn matching_library(&self) -> Vec<usize> {
        let query = self.search.to_lowercase();
        self.library
            .iter()
            .enumerate()
            .filter(|(_, item)| item.title.to_lowercase().contains(&query))
            .map(|(i, _)| i)
            .collect()
    }
    pub(crate) fn load_thumbnails(&mut self, ctx: &Context) {
        // Keep previews for the recent strip and current filtered page only.
        // Unloaded entries still have labeled, clickable controls in the library.
        let mut needed: Vec<usize> = (0..self.library.len().min(24)).collect();
        if self.workspace == Workspace::Library {
            needed.extend(
                self.matching_library()
                    .into_iter()
                    .skip(self.library_page * 24)
                    .take(24),
            );
        }
        for (i, item) in self.library.iter_mut().enumerate() {
            if !needed.contains(&i) {
                item.texture = None;
            }
        }
        if let Some(i) = needed
            .into_iter()
            .find(|&i| self.library[i].texture.is_none())
        {
            let item = &mut self.library[i];
            if let Ok(e) = Editor::open(&item.path) {
                let img = image::imageops::thumbnail(&e.render(), 240, 160);
                item.texture = Some(ctx.load_texture(
                    item.path.to_string_lossy(),
                    ColorImage::from_rgba_unmultiplied(
                        [img.width() as usize, img.height() as usize],
                        img.as_raw(),
                    ),
                    TextureOptions::LINEAR,
                ));
            } else {
                let img = ColorImage::filled([80, 50], CANVAS);
                item.texture = Some(ctx.load_texture(
                    item.path.to_string_lossy(),
                    img,
                    TextureOptions::LINEAR,
                ));
            }
            ctx.request_repaint();
        }
        if self.thumb_revision != self.editor.revision || self.current_thumb.is_none() {
            let img = image::imageops::thumbnail(&self.editor.render(), 240, 160);
            let color = ColorImage::from_rgba_unmultiplied(
                [img.width() as usize, img.height() as usize],
                img.as_raw(),
            );
            if let Some(t) = &mut self.current_thumb {
                t.set(color, TextureOptions::LINEAR);
            } else {
                self.current_thumb =
                    Some(ctx.load_texture("current preview", color, TextureOptions::LINEAR));
            }
            self.thumb_revision = self.editor.revision;
        }
        while self.guide_textures.len() < self.steps.len() {
            let i = self.guide_textures.len();
            let img = image::imageops::thumbnail(&self.steps[i].1, 640, 400);
            self.guide_textures.push(ctx.load_texture(
                format!("guide {} {}", i, nonce()),
                ColorImage::from_rgba_unmultiplied(
                    [img.width() as usize, img.height() as usize],
                    img.as_raw(),
                ),
                TextureOptions::LINEAR,
            ));
        }
    }
    pub(crate) fn header(&mut self, ctx: &Context) {
        TopBottomPanel::top("main navigation")
            .frame(
                egui::Frame::NONE
                    .fill(Color32::WHITE)
                    .inner_margin(Margin::symmetric(16, 10)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().interact_size.y = 34.;
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), 34.),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.set_height(34.);
                        ui.label(RichText::new("Framewell").size(22.).strong().color(ACCENT));
                        ui.add_space(16.);
                        for (mode, label) in [
                            (Workspace::Editor, "Editor"),
                            (Workspace::Library, "Library"),
                            (Workspace::Guide, "Guides"),
                        ] {
                            if ui
                                .add(
                                    Button::new(label)
                                        .selected(self.workspace == mode)
                                        .min_size(vec2(72., 34.)),
                                )
                                .clicked()
                            {
                                self.workspace = mode;
                            }
                        }
                        ui.allocate_ui_with_layout(
                            vec2(ui.available_width(), 34.),
                            Layout::right_to_left(Align::Center),
                            |ui| {
                                ui.set_height(34.);
                                if ui
                                    .add(
                                        Button::new(RichText::new("Capture").color(Color32::WHITE))
                                            .fill(ACCENT)
                                            .min_size(vec2(92., 34.)),
                                    )
                                    .clicked()
                                    && self.job.is_none()
                                {
                                    self.request_action(PendingAction::Capture, ctx);
                                }
                                ComboBox::from_id_salt("capture mode")
                                    .width(88.)
                                    .selected_text(&self.capture_mode)
                                    .show_ui(ui, |ui| {
                                        for mode in ["Region", "Window", "Display", "Scrolling"] {
                                            ui.selectable_value(
                                                &mut self.capture_mode,
                                                mode.into(),
                                                mode,
                                            );
                                        }
                                    });
                                if ui
                                    .add(Button::new("Export").min_size(vec2(74., 34.)))
                                    .clicked()
                                {
                                    if self.workspace == Workspace::Guide {
                                        self.export_guide_dialog();
                                    } else {
                                        self.export_png();
                                    }
                                }
                                if ui
                                    .add(Button::new("Save").min_size(vec2(66., 34.)))
                                    .clicked()
                                {
                                    if self.workspace == Workspace::Guide {
                                        self.save_guide();
                                    } else {
                                        self.save();
                                    }
                                }
                                if ui
                                    .add(Button::new("Import").min_size(vec2(70., 34.)))
                                    .clicked()
                                {
                                    self.request_action(PendingAction::Import, ctx);
                                }
                            },
                        );
                    },
                );
            });
    }
    pub(crate) fn tool_bar(&mut self, ctx: &Context) {
        TopBottomPanel::top("annotation shelf")
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(249, 250, 251))
                    .inner_margin(Margin::symmetric(16, 5)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.;
                    for tool in [
                        Tool::Select,
                        Tool::Arrow,
                        Tool::Text,
                        Tool::Callout,
                        Tool::Rectangle,
                        Tool::Step,
                        Tool::Pen,
                        Tool::Highlight,
                        Tool::Blur,
                        Tool::Redact,
                    ] {
                        if tool_button(ui, tool, self.tool == tool).clicked() {
                            self.tool = tool;
                            self.selected = None;
                            self.inspector = Inspector::Properties;
                            if matches!(tool, Tool::Text | Tool::Callout) {
                                self.width = 24.;
                            } else {
                                self.width = 4.;
                            }
                        }
                    }
                    ui.add_space(12.);
                    ui.separator();
                    ui.vertical(|ui| {
                        if ui.button("Undo").on_hover_text("Command Z").clicked() {
                            self.editor.undo();
                            self.selected = None;
                            self.changed();
                        }
                        if ui.button("Redo").on_hover_text("Shift Command Z").clicked() {
                            self.editor.redo();
                            self.selected = None;
                            self.changed();
                        }
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                Button::new("AI assistant")
                                    .selected(self.inspector == Inspector::AI),
                            )
                            .clicked()
                        {
                            self.inspector = Inspector::AI;
                        }
                    });
                });
            });
    }
    pub(crate) fn footer(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("status")
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(249, 250, 251))
                    .inner_margin(Margin::symmetric(16, 7)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status).size(12.).color(MUTED));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} × {} px",
                                self.editor.state.pixels.width(),
                                self.editor.state.pixels.height()
                            ))
                            .small()
                            .color(MUTED),
                        );
                        if self.workspace == Workspace::Editor {
                            if ui.button("Fit").clicked() {
                                self.zoom = 1.;
                            }
                            ui.add(
                                Slider::new(&mut self.zoom, 0.5..=2.)
                                    .show_value(false)
                                    .logarithmic(true),
                            );
                            ui.label(
                                RichText::new(format!("{:.0}% of fit", self.zoom * 100.))
                                    .small()
                                    .color(MUTED),
                            );
                        }
                    });
                });
            });
    }
    pub(crate) fn filmstrip(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("recent captures").exact_height(142.).frame(egui::Frame::NONE.fill(Color32::from_rgb(246,248,250)).inner_margin(Margin::symmetric(16,8))).show(ctx,|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Recent captures").strong().size(13.));
                if ui.small_button("View library").clicked() { self.workspace=Workspace::Library; }
                ui.with_layout(Layout::right_to_left(Align::Center),|ui| {
                    if ui.button("Add to guide").clicked() { self.add_step(); }
                });
            });
            ScrollArea::horizontal().show(ui,|ui| {
                ui.horizontal(|ui| {
                    if let Some(texture)=&self.current_thumb {
                        image_tile(ui,texture,&self.editor.state.document.title,vec2(136.,92.),true);
                    }
                    let mut open=None;
                    for item in self.library.iter().take(24) {
                        if self.project_path.as_ref()==Some(&item.path) { continue; }
                        if let Some(t)=&item.texture && image_tile(ui,t,&item.title,vec2(136.,92.),false).clicked() { open=Some(item.path.clone()); }
                    }
                    if self.library.is_empty() {
                        ui.add_space(10.); ui.label(RichText::new("Your saved captures will appear here.\nCapture or import an image, then choose Save.").size(13.).color(MUTED));
                    }
                    if let Some(path)=open { self.request_open(path); }
                });
            });
        });
    }
    pub(crate) fn properties(&mut self, ctx: &Context) {
        SidePanel::right("inspector")
            .exact_width(286.)
            .resizable(false)
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_rgb(249, 250, 251))
                    .inner_margin(14),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (tab, label) in [
                        (Inspector::Properties, "Properties"),
                        (Inspector::Layers, "Layers"),
                        (Inspector::AI, "AI"),
                    ] {
                        if ui.selectable_label(self.inspector == tab, label).clicked() {
                            self.inspector = tab;
                        }
                    }
                });
                ui.separator();
                ScrollArea::vertical()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .show(ui, |ui| match self.inspector {
                        Inspector::Properties => self.tool_properties(ui, ctx),
                        Inspector::Layers => self.layers(ui),
                        Inspector::AI => self.ai_panel(ui),
                    });
            });
    }
    fn tool_properties(&mut self, ui: &mut Ui, ctx: &Context) {
        let target = self
            .selected
            .and_then(|i| self.editor.state.document.objects.get(i))
            .map_or(self.tool, |o| o.tool);
        if matches!(target, Tool::Text | Tool::Callout | Tool::Step) {
            section(ui, &format!("{} text", label(target)));
            if ui
                .add(
                    TextEdit::multiline(&mut self.text)
                        .desired_width(f32::INFINITY)
                        .desired_rows(3)
                        .hint_text("Write an explanation…"),
                )
                .changed()
            {
                self.apply_style();
            }
        }
        section(ui, "Quick Styles");
        if target == Tool::Select {
            ui.label(
                RichText::new("Select an annotation, or choose a tool above to see its styles.")
                    .color(MUTED),
            );
            ui.add_space(12.);
            icon(
                ui.painter(),
                Rect::from_min_size(ui.cursor().min, vec2(40., 40.)),
                Tool::Select,
                ACCENT,
                2.,
            );
            ui.add_space(48.);
        } else {
            Grid::new("quick styles")
                .spacing(vec2(7., 7.))
                .show(ui, |ui| {
                    for (i, color) in COLORS.into_iter().enumerate() {
                        if style_button(ui, target, color, self.color == color, i).clicked() {
                            self.color = color;
                            self.width = if matches!(target, Tool::Text | Tool::Callout) {
                                if i >= 4 { 32. } else { 24. }
                            } else if i >= 4 {
                                6.
                            } else {
                                3.
                            };
                            self.apply_style();
                            self.style_editing = false;
                        }
                        if i % 4 == 3 {
                            ui.end_row();
                        }
                    }
                });
        }
        section(ui, &format!("{} Properties", label(target)));
        ui.label(RichText::new(hint(target)).size(13.).color(MUTED));
        ui.add_space(10.);
        if target != Tool::Select {
            let mut changed = false;
            if !matches!(target, Tool::Blur | Tool::Redact) {
                ui.horizontal(|ui| {
                    ui.label("Color");
                    changed |= ui.color_edit_button_srgba(&mut self.color).changed();
                });
            }
            if !matches!(target, Tool::Redact | Tool::Highlight) {
                let name = if matches!(target, Tool::Text | Tool::Callout | Tool::Step) {
                    "Text size"
                } else if target == Tool::Blur {
                    "Blur strength"
                } else {
                    "Width"
                };
                changed |= ui
                    .add(Slider::new(&mut self.width, 1.0..=48.).text(name))
                    .changed();
            }
            if changed {
                self.apply_style();
            }
            if !ctx.input(|i| i.pointer.any_down()) && !changed {
                self.style_editing = false;
            }
        }
        if self.selected.is_some() {
            ui.add_space(12.);
            ui.label(
                RichText::new("Editing the selected annotation")
                    .small()
                    .color(ACCENT),
            );
            if ui.button("Delete annotation").clicked() {
                self.delete_selected();
            }
        }
        section(ui, "Guide caption");
        if ui
            .add(
                TextEdit::multiline(&mut self.editor.state.document.caption)
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .hint_text("Explain this image in your guide…"),
            )
            .changed()
        {
            self.editor.revision += 1;
            self.modified = true;
            self.proposal = None;
        }
        if ui
            .add_sized(
                [ui.available_width(), 32.],
                Button::new("Add image to guide"),
            )
            .clicked()
        {
            self.add_step();
        }
    }
    fn layers(&mut self, ui: &mut Ui) {
        section(ui, "Annotations");
        if self.editor.state.document.objects.is_empty() {
            ui.label("Your annotations will appear here. Choose a tool and drag on the image.");
        }
        let mut chosen = None;
        for (i, o) in self.editor.state.document.objects.iter().enumerate().rev() {
            let name = if o.text.is_empty() {
                format!("{} {}", label(o.tool), o.id)
            } else {
                format!(
                    "{} · {}",
                    label(o.tool),
                    o.text.chars().take(24).collect::<String>()
                )
            };
            if ui
                .selectable_label(self.selected == Some(i), name)
                .clicked()
            {
                chosen = Some(i);
            }
        }
        if let Some(i) = chosen {
            self.selected = Some(i);
            self.sync_selection();
            self.tool = Tool::Select;
        }
        if self.selected.is_some() {
            ui.separator();
            if ui.button("Edit properties").clicked() {
                self.inspector = Inspector::Properties;
            }
            if ui.button("Delete annotation").clicked() {
                self.delete_selected();
            }
            ui.horizontal(|ui| {
                if ui.button("Bring forward").clicked()
                    && let Some(i) = self.selected
                    && i + 1 < self.editor.state.document.objects.len()
                {
                    self.editor.checkpoint();
                    self.editor.state.document.objects.swap(i, i + 1);
                    self.selected = Some(i + 1);
                    self.changed();
                }
                if ui.button("Send backward").clicked()
                    && let Some(i) = self.selected
                    && i > 0
                {
                    self.editor.checkpoint();
                    self.editor.state.document.objects.swap(i, i - 1);
                    self.selected = Some(i - 1);
                    self.changed();
                }
            });
        }
    }
    fn ai_panel(&mut self, ui: &mut Ui) {
        section(ui, "Explain this image");
        ui.label("Draft a clear explanation for a colleague, then review it before adding it to your guide.");
        ui.add_space(12.);
        ui.label("What should the explanation focus on?");
        ui.add(
            TextEdit::multiline(&mut self.focus)
                .desired_width(f32::INFINITY)
                .desired_rows(4),
        );
        if ui
            .add_enabled(
                self.job.is_none(),
                Button::new("Draft explanation").min_size(vec2(ui.available_width(), 34.)),
            )
            .clicked()
        {
            self.start_ai();
        }
        if self.job.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Working…");
            });
        }
        if let Some(proposal) = &mut self.proposal {
            section(ui, "Review the draft");
            ui.add(
                TextEdit::multiline(proposal)
                    .desired_width(f32::INFINITY)
                    .desired_rows(9),
            );
            if ui.button("Use as guide caption").clicked() {
                let text = proposal.clone();
                self.editor.checkpoint();
                self.editor.state.document.caption = text;
                self.proposal = None;
                self.modified = true;
                self.status = "AI caption applied · Review the guide before exporting".into();
            }
        }
        section(ui, "Connection");
        ui.label(RichText::new("Uses your existing Codex login. The rendered image is sent only when you request a draft. Account limits apply.").size(12.).color(MUTED));
        ui.add_space(10.);
        ui.label(
            RichText::new("JEV and image generation are planned for the Rust rebuild.")
                .size(12.)
                .color(MUTED),
        );
    }
    pub(crate) fn library_view(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Capture library");
            ui.label(RichText::new(format!("{} saved images", self.library.len())).color(MUTED));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        TextEdit::singleline(&mut self.search)
                            .hint_text("Search captures…")
                            .desired_width(260.),
                    )
                    .changed()
                {
                    self.library_page = 0;
                }
            });
        });
        ui.label(
            RichText::new(
                "Stored on this Mac · Open a capture to continue editing its annotations",
            )
            .color(MUTED),
        );
        let matching = self.matching_library();
        let pages = matching.len().div_ceil(24).max(1);
        self.library_page = self.library_page.min(pages - 1);
        ui.add_space(14.);
        ui.horizontal(|ui| {
            ui.label(format!("{} matching captures", matching.len()));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add_enabled(self.library_page + 1 < pages, Button::new("Next"))
                    .clicked()
                {
                    self.library_page += 1;
                }
                ui.label(format!("Page {} of {}", self.library_page + 1, pages));
                if ui
                    .add_enabled(self.library_page > 0, Button::new("Previous"))
                    .clicked()
                {
                    self.library_page -= 1;
                }
            });
        });
        ui.add_space(14.);
        if self.library.is_empty() {
            ui.heading("Your visual workspace starts with a capture");
            ui.label("Capture a region, window, or display. You can also import a PNG or JPEG. Save it to see it here.");
            if ui.button("Return to editor").clicked() {
                self.workspace = Workspace::Editor;
            }
        } else if matching.is_empty() {
            ui.label("No captures match your search. Try another title.");
        }
        let columns = (ui.available_width() / 236.).floor().max(1.) as usize;
        let mut open = None;
        ScrollArea::vertical().show(ui, |ui| {
            Grid::new("capture grid")
                .spacing(vec2(16., 18.))
                .show(ui, |ui| {
                    for (index, i) in matching
                        .into_iter()
                        .skip(self.library_page * 24)
                        .take(24)
                        .enumerate()
                    {
                        let item = &self.library[i];
                        let response = if let Some(texture) = &item.texture {
                            image_tile(
                                ui,
                                texture,
                                &item.title,
                                vec2(220., 176.),
                                self.project_path.as_ref() == Some(&item.path),
                            )
                        } else {
                            ui.add_sized(
                                [220., 176.],
                                Button::new(format!("{}\nPreview loading…", item.title)),
                            )
                        };
                        if response.clicked() {
                            open = Some(item.path.clone());
                        }
                        if (index + 1) % columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
        if let Some(path) = open {
            self.request_open(path);
        }
    }
    fn add_step(&mut self) {
        let budget: u64 = self
            .steps
            .iter()
            .map(|(_, p)| u64::from(p.width()) * u64::from(p.height()))
            .sum();
        let current = u64::from(self.editor.state.pixels.width())
            * u64::from(self.editor.state.pixels.height());
        if self.steps.len() >= 50 || budget + current > 50_000_000 {
            self.status = "Guide is full · Export this guide before starting another".into();
            return;
        }
        self.steps.push((
            self.editor.state.document.caption.clone(),
            self.editor.render(),
        ));
        self.guide_modified = true;
        self.status = format!(
            "Added step {} · Open Guides to edit and arrange it",
            self.steps.len()
        );
    }
    pub(crate) fn save_guide(&mut self) {
        match guide::save(&self.steps, &self.root.join("guide")) {
            Ok(()) => {
                self.guide_modified = false;
                self.status = "Guide saved on this Mac".into();
            }
            Err(e) => self.status = format!("Could not save guide: {e}"),
        }
    }
    fn export_guide_dialog(&mut self) {
        if self.steps.is_empty() {
            self.status = "Add an image before exporting your guide".into();
            return;
        }
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            match export_guide(
                &self.steps,
                &path.join(format!("framewell-guide-{}", nonce())),
            ) {
                Ok(()) => self.status = "Guide exported as HTML with images".into(),
                Err(e) => self.status = format!("Guide export failed: {e}"),
            }
        }
    }
    fn export_png(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG", &["png"])
            .set_file_name("framewell.png")
            .save_file()
        {
            match self.editor.render().save(path) {
                Ok(()) => self.status = "PNG exported with opaque redactions".into(),
                Err(e) => self.status = format!("PNG export failed: {e}"),
            }
        }
    }
    pub(crate) fn guide_view(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Visual guide");
            ui.label(RichText::new(format!("{} steps", self.steps.len())).color(MUTED));
            if self.guide_modified {
                ui.label(RichText::new("Unsaved").color(MUTED));
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Add current image").clicked() {
                    self.add_step();
                }
            });
        });
        ui.label(RichText::new("Arrange your images, write clear instructions, then export a guide your colleagues can open in a browser.").color(MUTED));
        ui.add_space(22.);
        if self.steps.is_empty() {
            ui.heading("Show the process, one image at a time");
            ui.label("Annotate an image in the editor, then choose Add to guide. Each step keeps a snapshot of your annotations.");
            if ui.button("Go to editor").clicked() {
                self.workspace = Workspace::Editor;
            }
        }
        let mut move_step = None;
        let mut remove = None;
        ScrollArea::vertical().show(ui, |ui| {
            for (i, (caption, _)) in self.steps.iter_mut().enumerate() {
                egui::Frame::NONE
                    .fill(Color32::WHITE)
                    .corner_radius(6)
                    .inner_margin(18)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.set_width(300.);
                                if let Some(t) = self.guide_textures.get(i) {
                                    ui.add(egui::Image::new(t).max_size(vec2(300., 190.)));
                                }
                            });
                            ui.add_space(20.);
                            ui.vertical(|ui| {
                                ui.set_width((ui.available_width() - 12.).max(200.));
                                ui.label(
                                    RichText::new(format!("Step {}", i + 1)).size(18.).strong(),
                                );
                                if ui
                                    .add(
                                        TextEdit::multiline(caption)
                                            .desired_width(f32::INFINITY)
                                            .desired_rows(5)
                                            .hint_text("What should your colleague do here?"),
                                    )
                                    .changed()
                                {
                                    self.guide_modified = true;
                                }
                                ui.horizontal(|ui| {
                                    if ui.add_enabled(i > 0, Button::new("Move up")).clicked() {
                                        move_step = Some((i, i - 1));
                                    }
                                    if ui
                                        .add_enabled(
                                            i + 1 < self.guide_textures.len(),
                                            Button::new("Move down"),
                                        )
                                        .clicked()
                                    {
                                        move_step = Some((i, i + 1));
                                    }
                                    if ui.button("Remove").clicked() {
                                        remove = Some(i);
                                    }
                                });
                            });
                        });
                    });
                ui.add_space(14.);
            }
        });
        if let Some((a, b)) = move_step {
            self.steps.swap(a, b);
            if a < self.guide_textures.len() && b < self.guide_textures.len() {
                self.guide_textures.swap(a, b);
            } else {
                self.guide_textures.clear();
            }
            self.guide_modified = true;
        }
        if let Some(i) = remove {
            self.steps.remove(i);
            if i < self.guide_textures.len() {
                let _ = self.guide_textures.remove(i);
            }
            self.guide_modified = true;
        }
    }
}
