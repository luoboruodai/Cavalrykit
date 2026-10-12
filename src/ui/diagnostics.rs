//! Explicit opt-in, bounded native motion tracing. No names, search text or file paths
//! are recorded. File writes run on a separate thread; tracing never requests frames.
use std::{
    io::Write,
    sync::{
        mpsc::{self, SyncSender},
        OnceLock,
    },
    time::{Duration, Instant},
};
enum Message {
    Text(String),
    Frame(u64, std::sync::Arc<egui::ColorImage>),
}
struct Trace {
    tx: SyncSender<Message>,
    frames: bool,
    started: Instant,
}
fn trace() -> Option<&'static Trace> {
    static TRACE: OnceLock<Option<Trace>> = OnceLock::new();
    TRACE
        .get_or_init(|| {
            let path = std::env::var_os("CAVALRYKIT_MOTION_TRACE").or_else(|| {
                #[cfg(debug_assertions)]
                {
                    option_env!("CAVALRYKIT_MOTION_TRACE").map(std::ffi::OsString::from)
                }
                #[cfg(not(debug_assertions))]
                {
                    None
                }
            })?;
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .ok()?;
            let frame_dir = std::env::var_os("CAVALRYKIT_MOTION_FRAMES")
                .or_else(|| {
                    #[cfg(debug_assertions)]
                    {
                        option_env!("CAVALRYKIT_MOTION_FRAMES").map(std::ffi::OsString::from)
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        None
                    }
                })
                .map(std::path::PathBuf::from)
                .filter(|p| p.is_dir());
            let frames = frame_dir.is_some();
            let (tx, rx) = mpsc::sync_channel::<Message>(4096);
            std::thread::Builder::new()
                .name("motion-trace".into())
                .spawn(move || {
                    let mut file = std::io::BufWriter::new(file);
                    for message in rx.iter().take(100_000) {
                        match message {
                            Message::Text(line) => {
                                let _ = writeln!(file, "{line}");
                                let _ = file.flush();
                            }
                            Message::Frame(n, image) => {
                                if let Some(dir) = &frame_dir {
                                    let bytes: Vec<u8> =
                                        image.pixels.iter().flat_map(|c| c.to_array()).collect();
                                    let _ = image::save_buffer(
                                        dir.join(format!("native-{n:06}.png")),
                                        &bytes,
                                        image.size[0] as u32,
                                        image.size[1] as u32,
                                        image::ColorType::Rgba8,
                                    );
                                }
                            }
                        }
                    }
                })
                .ok()?;
            Some(Trace {
                tx,
                frames,
                started: Instant::now(),
            })
        })
        .as_ref()
        .filter(|t| t.started.elapsed() < Duration::from_secs(300))
}
pub fn sample(ctx: &egui::Context, kind: &str, id: egui::Id, target: f32, value: f32) {
    let Some(trace) = trace() else { return };
    let (time, dt, stable, events, hovered) = ctx.input(|i| {
        (
            i.time,
            i.unstable_dt,
            i.stable_dt,
            i.events.len(),
            i.raw.hovered_files.len(),
        )
    });
    if (target - value).abs() > 0.001 {
        let pass = ctx.cumulative_pass_nr();
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("motion-active-pass"), pass));
    }
    // Record changes, not every settled widget on every frame. The cap therefore
    // covers several minutes of interaction rather than filling on a large list.
    let changed = ctx.data_mut(|d| {
        let key = egui::Id::new(("trace-sample", kind, id));
        let old = d.get_temp::<(f32, f32)>(key);
        let different = old.is_none_or(|(t, v)| t != target || (v - value).abs() > 0.001);
        if different {
            d.insert_temp(key, (target, value));
        }
        different
    });
    if !changed {
        return;
    }
    let line=serde_json::json!({"kind":kind,"id":format!("{id:?}"),"time":time,"dt":dt,"stable_dt":stable,"pass":ctx.cumulative_pass_nr(),"events":events,"hovered_files":hovered,"target":target,"value":value,"animation_time":ctx.style().animation_time}).to_string();
    let _ = trace.tx.try_send(Message::Text(line));
}

#[derive(Clone, Copy, Default)]
struct Capture {
    last: f64,
    count: u64,
}
/// Optional rendered native-window captures. Do not enable this for timing benchmarks:
/// screenshot readback itself can trigger frames, so trace-only validation comes first.
pub fn frame(ctx: &egui::Context) {
    let Some(trace) = trace() else { return };
    if !trace.frames {
        return;
    }
    ctx.input(|i| {
        for e in &i.events {
            if let egui::Event::Screenshot {
                user_data, image, ..
            } = e
            {
                if let Some(n) = user_data
                    .data
                    .as_ref()
                    .and_then(|d| d.downcast_ref::<u64>())
                {
                    let _ = trace.tx.try_send(Message::Frame(*n, image.clone()));
                }
            }
        }
    });
    let pass = ctx.cumulative_pass_nr();
    let now = ctx.input(|i| i.time);
    let active = ctx.data(|d| d.get_temp::<u64>(egui::Id::new("motion-active-pass"))) == Some(pass);
    let capture = ctx.data_mut(|d| {
        let id = egui::Id::new("motion-capture-state");
        let mut s = d.get_temp::<Capture>(id).unwrap_or_default();
        if active && now - s.last >= 0.05 && now > 1.0 && s.count < 160 {
            s.count += 1;
            s.last = now;
            d.insert_temp(id, s);
            Some(s.count)
        } else {
            None
        }
    });
    if let Some(n) = capture {
        let _ = trace.tx.try_send(Message::Text(
            serde_json::json!({"kind":"capture","frame":n,"time":now,"pass":pass}).to_string(),
        ));
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(n)));
    }
}
