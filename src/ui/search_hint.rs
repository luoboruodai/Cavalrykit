//! Slow, occasional suggestions. Never replace user text or keep an idle animation loop running.
use crate::i18n::Key as K;
use egui::{Context, Id};
use std::time::Duration;
const DEFAULT_SECONDS: f64 = 20.0;
const TIP_SECONDS: f64 = 4.0;
const FADE_SECONDS: f64 = 0.24;
const TIPS: [K; 16] = [
    K::SearchTip1,
    K::SearchTip2,
    K::SearchTip3,
    K::SearchTip4,
    K::SearchTip5,
    K::SearchTip6,
    K::SearchTip7,
    K::SearchTip8,
    K::SearchTip9,
    K::SearchTip10,
    K::SearchTip11,
    K::SearchTip12,
    K::SearchTip13,
    K::SearchTip14,
    K::SearchTip15,
    K::SearchTip16,
];
#[derive(Clone, Copy)]
pub struct Hint {
    pub key: K,
    pub opacity: f32,
    pub offset: f32,
}
/// The default occupies >80% of the cycle. Only the two short transitions request frames.
fn phase(elapsed: f64) -> (Hint, Duration) {
    let cycle = DEFAULT_SECONDS + TIP_SECONDS;
    let elapsed = elapsed.max(0.0);
    let local = elapsed % cycle;
    let tip = TIPS[((elapsed / cycle) as usize) % TIPS.len()];
    let (key, start, end) = if local < DEFAULT_SECONDS {
        (K::Search, 0.0, DEFAULT_SECONDS)
    } else {
        (tip, DEFAULT_SECONDS, cycle)
    };
    let entering = if elapsed < FADE_SECONDS {
        1.0
    } else {
        ((local - start) / FADE_SECONDS).clamp(0.0, 1.0)
    };
    let leaving = ((end - local) / FADE_SECONDS).clamp(0.0, 1.0);
    let opacity = entering.min(leaving) as f32;
    let delay = if opacity < 1.0 {
        Duration::ZERO
    } else {
        Duration::from_secs_f64((end - local - FADE_SECONDS).max(0.0))
    };
    (
        Hint {
            key,
            opacity,
            offset: (1.0 - opacity) * if entering < leaving { 4.0 } else { -4.0 },
        },
        delay,
    )
}
pub fn current(ctx: &Context, eligible: bool) -> Hint {
    let id = Id::new("idle-search-hint-clock");
    if !eligible || ctx.style().animation_time == 0.0 {
        ctx.data_mut(|d| d.remove::<f64>(id));
        return Hint {
            key: K::Search,
            opacity: 1.0,
            offset: 0.0,
        };
    }
    let now = ctx.input(|i| i.time);
    let since = ctx.data_mut(|d| *d.get_temp_mut_or_insert_with(id, || now));
    let (hint, delay) = phase(now - since);
    ctx.request_repaint_after(delay);
    hint
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_dominates_and_all_sixteen_tips_are_reachable_without_frame_polling() {
        for (i, key) in TIPS.iter().enumerate() {
            let (default, delay) = phase(i as f64 * 24.0 + 5.0);
            assert_eq!(default.key, K::Search);
            assert!(delay > Duration::from_secs(14));
            let (tip, delay) = phase(i as f64 * 24.0 + 21.0);
            assert_eq!(tip.key, *key);
            assert_eq!(tip.opacity, 1.0);
            assert!(delay > Duration::from_secs(2));
        }
        assert_eq!(phase(19.9).1, Duration::ZERO);
        assert_eq!(phase(20.1).1, Duration::ZERO);
    }
    #[test]
    fn focus_modal_typing_and_reduce_motion_reset_the_cycle() {
        let ctx = Context::default();
        for (time, eligible, reduced, expected) in [
            (0.0, true, false, K::Search),
            (21.0, true, false, K::SearchTip1),
            (22.0, false, false, K::Search),
            (23.0, true, false, K::Search),
            (45.0, true, true, K::Search),
            (46.0, true, false, K::Search),
        ] {
            ctx.style_mut(|s| s.animation_time = if reduced { 0.0 } else { 0.18 });
            let output = ctx.run(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ctx| assert_eq!(current(ctx, eligible).key, expected),
            );
            if !eligible || reduced {
                assert!(
                    output.viewport_output[&egui::ViewportId::ROOT].repaint_delay
                        > Duration::from_secs(1)
                );
            }
        }
    }
}
