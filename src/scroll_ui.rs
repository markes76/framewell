use super::*;
use framewell::scrolling::Panorama;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU8, Ordering},
};

pub(crate) struct ScrollPreview {
    pub image: image::RgbaImage,
    pub texture: TextureHandle,
    pub bounds: [f64; 4],
    pub start: Option<Pos2>,
    pub area: Option<Rect>,
    pub error: String,
}
pub(crate) struct ScrollRun {
    stop: Arc<AtomicU8>,
    progress: Arc<Mutex<(usize, String)>>,
    position: [f32; 2],
}
impl Drop for ScrollRun {
    fn drop(&mut self) {
        self.stop.store(2, Ordering::SeqCst);
    }
}
impl Framewell {
    pub(crate) fn scrolling_ui(&mut self, ctx: &Context) {
        let mut begin = None;
        let mut cancel = false;
        if let Some(preview) = &mut self.scroll_preview {
            let modal = egui::Modal::new(Id::new("scrolling region selection")).show(ctx,|ui| {
                ui.heading("Select scrolling content");
                ui.label("Drag over the scrolling content in this preview. Exclude fixed headers, borders and scrollbars.");
                ui.label(RichText::new("Then scroll downward slowly in the real window. Choose Finish when you have captured enough.").color(MUTED));
                ui.add_space(8.);
                let available=vec2((ctx.content_rect().width()-100.).min(900.),(ctx.content_rect().height()-230.).max(220.));
                let dimensions=vec2(preview.image.width() as f32,preview.image.height() as f32);
                let scale=(available.x/dimensions.x).min(available.y/dimensions.y);
                let (r,response)=ui.allocate_exact_size(dimensions*scale,Sense::click_and_drag());
                ui.painter().image(preview.texture.id(),r,Rect::from_min_max(Pos2::ZERO,pos2(1.,1.)),Color32::WHITE);
                let point=|p:Pos2|pos2(((p.x-r.left())/scale).clamp(0.,dimensions.x),((p.y-r.top())/scale).clamp(0.,dimensions.y));
                if response.drag_started() { preview.start=drag_press_origin(ui,&response).map(point); }
                if (response.dragged() || response.drag_stopped()) && let (Some(start),Some(p))=(preview.start,response.interact_pointer_pos()) {
                    preview.area=Some(Rect::from_two_pos(start,point(p)));
                }
                if let Some(area)=preview.area {
                    let mark=Rect::from_min_max(r.min+area.min.to_vec2()*scale,r.min+area.max.to_vec2()*scale);
                    ui.painter().rect_stroke(mark,0,Stroke::new(2.0_f32,ACCENT),StrokeKind::Inside);
                    ui.label(format!("Selected {:.0} × {:.0} pixels",area.width(),area.height()));
                } else { ui.label("Select an area in the image above."); }
                if !preview.error.is_empty() { ui.label(RichText::new(&preview.error).strong().color(INK)); }
                ui.horizontal(|ui| {
                    if ui.add_enabled(preview.area.is_some_and(|r|r.width()>=32. && r.height()>=32.),Button::new("Start scrolling capture")).clicked() {
                        let area=preview.area.unwrap();
                        let size=[preview.image.width(),preview.image.height()];
                        let region=platform::preview_capture_region([area.left(),area.top(),area.width(),area.height()],preview.bounds,size);
                        begin=Some((region,preview.bounds,size));
                    }
                    if ui.button("Cancel").clicked() { cancel=true; }
                });
            });
            cancel |= modal.should_close();
        }
        if cancel {
            self.scroll_preview = None;
            self.status = "Scrolling capture cancelled. Your image is unchanged.".into();
        }
        if let Some((region, bounds, size)) = begin {
            self.start_scrolling(ctx, region, bounds, size);
        }
        if let Some(run) = &self.scroll_run {
            ctx.show_viewport_immediate(
                ViewportId::from_hash_of("scrolling controls"),
                ViewportBuilder::default()
                    .with_title("Framewell · Scrolling capture")
                    .with_inner_size([380., 120.])
                    .with_position(run.position)
                    .with_resizable(false)
                    .with_window_level(WindowLevel::AlwaysOnTop),
                |ctx, _| {
                    if ctx.input(|i| i.viewport().close_requested()) {
                        run.stop.store(2, Ordering::SeqCst);
                    }
                    CentralPanel::default().show(ctx, |ui| {
                        let (frames, message) = run.progress.lock().unwrap().clone();
                        ui.label(
                            RichText::new(format!(
                                "Scroll downward slowly · {frames} {}",
                                if frames == 1 { "frame" } else { "frames" }
                            ))
                            .strong(),
                        );
                        ui.label(message);
                        ui.horizontal(|ui| {
                            if ui.button("Finish").clicked() {
                                run.stop.store(1, Ordering::SeqCst);
                            }
                            if ui.button("Cancel").clicked() {
                                run.stop.store(2, Ordering::SeqCst);
                            }
                        });
                    });
                    ctx.request_repaint_after(Duration::from_millis(150));
                },
            );
        }
    }
    fn start_scrolling(
        &mut self,
        ctx: &Context,
        region: [f64; 4],
        bounds: [f64; 4],
        size: [u32; 2],
    ) {
        // Geometry only: useful for diagnosing display scaling; no captured pixels or titles.
        let _ = std::fs::create_dir_all(&self.root);
        let _=std::fs::write(self.root.join("capture-diagnostics.json"),serde_json::to_vec_pretty(&serde_json::json!({"preview_pixels":size,"display_bounds":bounds,"capture_rect":region})).unwrap_or_default());
        let sx = bounds[2] / f64::from(size[0]);
        let sy = bounds[3] / f64::from(size[1]);
        let selected = Rect::from_min_size(
            pos2(
                (bounds[0] + region[0] * sx) as f32,
                (bounds[1] + region[1] * sy) as f32,
            ),
            vec2((region[2] * sx) as f32, (region[3] * sy) as f32),
        );
        let positions = [
            [
                (bounds[0] + 20.) as f32,
                (bounds[1] + bounds[3] - 220.) as f32,
            ],
            [
                (bounds[0] + bounds[2] - 440.) as f32,
                (bounds[1] + bounds[3] - 220.) as f32,
            ],
            [(bounds[0] + 20.) as f32, (bounds[1] + 32.) as f32],
            [
                (bounds[0] + bounds[2] - 440.) as f32,
                (bounds[1] + 32.) as f32,
            ],
        ];
        let Some(position) = positions.into_iter().find(|p| {
            !Rect::from_min_size(pos2(p[0], p[1]), vec2(420., 180.))
                .expand(20.)
                .intersects(selected)
        }) else {
            if let Some(preview) = &mut self.scroll_preview {
                preview.error =
                    "Choose a smaller area so the capture controls can stay outside it.".into();
            }
            return;
        };
        self.scroll_preview = None;
        let stop = Arc::new(AtomicU8::new(0));
        let progress = Arc::new(Mutex::new((
            0,
            "Keep at least 20% overlap between frames.".into(),
        )));
        self.scroll_run = Some(ScrollRun {
            stop: stop.clone(),
            progress: progress.clone(),
            position,
        });
        let (tx, rx) = mpsc::channel();
        self.job = Some(rx);
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        let wake = ctx.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(500));
            let result = (|| -> framewell::Result<Option<image::RgbaImage>> {
                let mut panorama = None;
                let mut failures = 0;
                let mut frames = 0;
                let start = std::time::Instant::now();
                loop {
                    match stop.load(Ordering::SeqCst) {
                        2 => return Ok(None),
                        1 => {
                            return panorama
                                .map(Panorama::finish)
                                .map(Some)
                                .ok_or_else(|| "No image was captured. Try again.".into());
                        }
                        _ => {}
                    }
                    let path = platform::capture_region(region)?;
                    let image = load_image(&path);
                    let _ = std::fs::remove_file(path);
                    let image = image?;
                    if let Some(p) = &mut panorama {
                        match p.append(image) {
                            Ok(changed) => {
                                failures = 0;
                                if changed {
                                    frames += 1;
                                }
                            }
                            Err(e) => {
                                failures += 1;
                                progress.lock().unwrap().1 = e.to_string();
                                if failures >= 3 {
                                    return Err(e);
                                }
                            }
                        }
                    } else {
                        panorama = Some(Panorama::new(image)?);
                        frames = 1;
                    }
                    if failures == 0 {
                        *progress.lock().unwrap() = (
                            frames,
                            "Scroll the real window; these controls stay outside the area.".into(),
                        );
                    }
                    if frames >= 80 || start.elapsed() > Duration::from_secs(120) {
                        return Ok(panorama.map(Panorama::finish));
                    }
                    std::thread::sleep(Duration::from_millis(850));
                }
            })();
            let _ = tx.send(Job::Scrolling(result.map_err(|e| e.to_string())));
            wake.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Visible(true));
            wake.request_repaint_of(ViewportId::ROOT);
        });
    }
}
