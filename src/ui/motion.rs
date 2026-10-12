//! Finite, interruptible UI choreography. Never drives a timer while settled.
use egui::{Context, Id};

#[derive(Clone, Copy)]
struct Reveal {
    key: Id,
    started: f64,
    last_pass: u64,
}

/// Restart when the content key changes or the surface has been absent. A new
/// action replaces the old timeline; no animation task or stale callback survives.
pub fn reveal(ctx: &Context, slot: Id, key: Id, seconds: f32) -> f32 {
    let now = ctx.input(|i| i.time);
    let pass = ctx.cumulative_pass_nr();
    let state = ctx.data_mut(|data| {
        let mut state = data.get_temp::<Reveal>(slot).unwrap_or(Reveal {
            key,
            started: now,
            last_pass: pass,
        });
        if state.key != key || pass > state.last_pass + 1 {
            state = Reveal {
                key,
                started: now,
                last_pass: pass,
            };
        }
        state.last_pass = pass;
        data.insert_temp(slot, state);
        state
    });
    if ctx.style().animation_time == 0.0 {
        return 1.0;
    }
    let progress = ((now - state.started) as f32 / seconds.max(0.001)).clamp(0.0, 1.0);
    if progress < 1.0 {
        ctx.request_repaint();
    }
    super::diagnostics::sample(ctx, "reveal", slot, 1.0, progress);
    progress
}
#[derive(Clone, Copy)]
struct Travel {
    from: f32,
    to: f32,
    started: f64,
}
/// Ease from the current painted position when the user reverses a selection mid-flight.
pub fn travel(ctx: &Context, id: Id, target: f32, seconds: f32) -> f32 {
    let now = ctx.input(|i| i.time);
    let immediate = ctx.style().animation_time == 0.0;
    let sample = |s: Travel| {
        let k = cubic_out(((now - s.started) as f32 / seconds.max(0.001)).clamp(0.0, 1.0));
        s.from + (s.to - s.from) * k
    };
    let state = ctx.data_mut(|data| {
        let mut state = data.get_temp::<Travel>(id).unwrap_or(Travel {
            from: target,
            to: target,
            started: now,
        });
        if state.to != target {
            state = Travel {
                from: sample(state),
                to: target,
                started: now,
            };
        }
        if immediate {
            state = Travel {
                from: target,
                to: target,
                started: now,
            };
        }
        data.insert_temp(id, state);
        state
    });
    if state.from != state.to && now - state.started < seconds as f64 {
        ctx.request_repaint();
    }
    let value = sample(state);
    super::diagnostics::sample(ctx, "travel", id, target, value);
    value
}
#[derive(Clone, Copy)]
struct ModalMotion {
    closing: bool,
    started: f64,
    from: f32,
}
pub fn modal_progress(ctx: &Context, id: Id, closing: bool, seconds: f32) -> f32 {
    let now = ctx.input(|i| i.time);
    let target = if closing { 0.0 } else { 1.0 };
    let state = ctx.data_mut(|d| {
        let key = id.with("modal-motion");
        let mut s = d.get_temp::<ModalMotion>(key).unwrap_or(ModalMotion {
            closing: false,
            started: now,
            from: 0.0,
        });
        if s.closing != closing {
            let old = if s.closing { 0.0 } else { 1.0 };
            s = ModalMotion {
                closing,
                started: now,
                from: old,
            };
        }
        d.insert_temp(key, s);
        s
    });
    if ctx.style().animation_time == 0.0 {
        return target;
    }
    let k = cubic_out(((now - state.started) as f32 / seconds.max(0.001)).clamp(0.0, 1.0));
    let value = state.from + (target - state.from) * k;
    if (value - target).abs() > 0.001 {
        ctx.request_repaint();
    }
    value
}
pub fn cubic_out(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}
pub fn phase(t: f32, start: f32, end: f32) -> f32 {
    cubic_out(((t - start) / (end - start)).clamp(0.0, 1.0))
}
/// Small decorative settle only. Do not apply this to hit regions or layout size.
pub fn settle(t: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let x = t - 1.0;
    1.0 + 2.1 * x.powi(3) + 1.1 * x.powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reveal_restarts_when_interrupted_or_reopened_and_stops_at_the_end() {
        let ctx = Context::default();
        let mut last = 0.0;
        for (frame, key, show) in [
            (0, 1, true),
            (1, 1, true),
            (2, 2, true),
            (3, 2, true),
            (4, 2, false),
            (5, 2, true),
            (6, 2, true),
            (7, 2, true),
            (8, 2, true),
        ] {
            let output = ctx.run(
                egui::RawInput {
                    time: Some(frame as f64),
                    ..Default::default()
                },
                |ctx| {
                    if show {
                        last = reveal(ctx, Id::new("surface"), Id::new(key), 0.3);
                    }
                },
            );
            if [0, 2, 5].contains(&frame) {
                assert_eq!(last, 0.0);
            }
            if [1, 3, 6].contains(&frame) {
                assert_eq!(last, 1.0);
            }
            if frame == 8 {
                assert_eq!(
                    output.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
                    std::time::Duration::MAX
                );
            }
        }
        ctx.style_mut(|s| s.animation_time = 0.0);
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            assert_eq!(reveal(ctx, Id::new("surface"), Id::new(3), 0.3), 1.0)
        });
    }
    #[test]
    fn selection_travel_retargets_without_a_jump_and_respects_reduce_motion() {
        let ctx = Context::default();
        let id = Id::new("selection-travel");
        let mut values = Vec::new();
        for (time, target) in [
            (0.0, 0.0),
            (0.1, 100.0),
            (0.2, 100.0),
            (0.2, 0.0),
            (0.8, 0.0),
        ] {
            let _ = ctx.run(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ctx| values.push(travel(ctx, id, target, 0.3)),
            );
        }
        assert_eq!(values[0], 0.0);
        assert_eq!(values[1], 0.0);
        assert!(values[2] > 0.0 && values[2] < 100.0);
        assert_eq!(
            values[2], values[3],
            "reversing direction begins at the displayed position"
        );
        assert_eq!(values[4], 0.0);
        ctx.style_mut(|s| s.animation_time = 0.0);
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            assert_eq!(travel(ctx, id, 100.0, 0.3), 100.0)
        });
    }
    #[test]
    fn phased_reveal_and_decorative_settle_have_exact_terminal_states() {
        assert_eq!(phase(0.0, 0.15, 1.0), 0.0);
        assert_eq!(phase(1.0, 0.15, 1.0), 1.0);
        assert_eq!(settle(0.0), 0.0);
        assert_eq!(settle(1.0), 1.0);
        assert!(
            (0..100)
                .map(|i| settle(i as f32 / 100.0))
                .fold(0.0_f32, f32::max)
                < 1.05
        );
    }
}
