//! Small, consistent vector icons. Never depend on symbol-font coverage.
use egui::{Color32, Painter, Rect, Stroke, Vec2};
#[derive(Clone, Copy, Debug)]
pub enum Icon {
    More,
    Check,
    Library,
    Script,
    Plugin,
    Folder,
    Reveal,
    Info,
    Settings,
    Search,
    Refresh,
    Trash,
    Import,
    Close,
    Display,
    Palette,
    AppLocation,
}
pub fn paint(p: &Painter, rect: Rect, icon: Icon, color: Color32) {
    paint_rotated(p, rect, icon, color, 0.0);
}
pub fn paint_rotated(p: &Painter, rect: Rect, icon: Icon, color: Color32, angle: f32) {
    let c = rect.center();
    let scale = rect.width().min(rect.height()) / 20.0;
    let (sin, cos) = angle.sin_cos();
    let point = |x, y| {
        let x = x * scale;
        let y = y * scale;
        c + Vec2::new(x * cos - y * sin, x * sin + y * cos)
    };
    let stroke = Stroke::new(1.4 * scale, color);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([point(a.0, a.1), point(b.0, b.1)], stroke);
    };
    let box_at = |x, y, w, h| {
        let rect = Rect::from_center_size(point(x, y), Vec2::new(w, h) * scale);
        if angle == 0.0 {
            p.rect_stroke(rect, 2, stroke, egui::StrokeKind::Inside);
        } else {
            rect_rotated(p, rect, 2.0, angle, Color32::TRANSPARENT, stroke);
        }
    };
    match icon {
        Icon::More => {
            for y in [-5.0, 0.0, 5.0] {
                p.circle_filled(point(0.0, y), 1.3 * scale, color);
            }
        }
        Icon::Check => {
            line((-6.0, 0.0), (-1.5, 4.5));
            line((-1.5, 4.5), (7.0, -5.0));
        }
        Icon::Library => {
            for (x, y) in [(-4.5, -4.5), (4.5, -4.5), (-4.5, 4.5), (4.5, 4.5)] {
                box_at(x, y, 6.0, 6.0);
            }
        }
        Icon::Script => {
            box_at(0.0, 0.0, 13.0, 16.0);
            for y in [-3.0, 1.0_f32, 5.0] {
                line((-3.0, y), (3.0, y));
            }
        }
        Icon::Plugin => {
            // A single module outline, rather than overlapping circles and a square.
            p.add(egui::Shape::closed_line(
                [
                    point(0.0, -8.0),
                    point(7.0, -4.0),
                    point(7.0, 4.0),
                    point(0.0, 8.0),
                    point(-7.0, 4.0),
                    point(-7.0, -4.0),
                ]
                .to_vec(),
                stroke,
            ));
            line((-7.0, -4.0), (0.0, 0.0));
            line((0.0, 0.0), (7.0, -4.0));
            line((0.0, 0.0), (0.0, 8.0));
        }
        Icon::Folder => {
            p.add(egui::Shape::closed_line(
                [
                    point(-8.0, -6.0),
                    point(-2.0, -6.0),
                    point(0.0, -3.0),
                    point(8.0, -3.0),
                    point(8.0, 7.0),
                    point(-8.0, 7.0),
                ]
                .to_vec(),
                stroke,
            ));
        }
        Icon::Reveal => {
            // Same-size open/reveal action as Trash; no mixed text/icon row actions.
            line((-6.0, -2.0), (-6.0, 7.0));
            line((-6.0, 7.0), (5.0, 7.0));
            line((5.0, 7.0), (5.0, 3.0));
            line((-6.0, -2.0), (-2.0, -2.0));
            line((0.0, -7.0), (7.0, -7.0));
            line((7.0, -7.0), (7.0, 0.0));
            line((-1.0, 1.0), (7.0, -7.0));
        }
        Icon::Info => {
            p.circle_stroke(c, 8.0 * scale, stroke);
            p.circle_filled(point(0.0, -3.5), 0.9 * scale, color);
            line((0.0, -0.5), (0.0, 4.5));
        }
        Icon::Settings => {
            for y in [-5.0, 0.0, 5.0] {
                line((-7.0, y), (7.0, y));
            }
            for (x, y) in [(-3.0, -5.0), (3.0, 0.0), (-2.0_f32, 5.0)] {
                p.circle_filled(point(x, y), 2.0_f32 * scale, color);
            }
        }
        Icon::Search => {
            p.circle_stroke(point(-1.5, -1.5), 5.5 * scale, stroke);
            line((3.0, 3.0), (7.0, 7.0));
        }
        Icon::Refresh => paint_refresh(p, rect, color, 0.0),
        Icon::Trash => {
            line((-7.0, -5.0), (7.0, -5.0));
            line((-3.0, -8.0), (3.0, -8.0));
            line((-5.0, -3.0), (-4.0, 7.0));
            line((5.0, -3.0), (4.0, 7.0));
            line((-4.0, 7.0), (4.0, 7.0));
            line((-1.5, -1.0_f32), (-1.5, 4.5));
            line((1.5, -1.0_f32), (1.5, 4.5));
        }
        Icon::Import => {
            line((0.0, -8.0), (0.0, 3.0));
            line((-4.0, -1.0_f32), (0.0, 3.0));
            line((4.0, -1.0_f32), (0.0, 3.0));
            line((-7.0, 3.0), (-7.0, 8.0));
            line((-7.0, 8.0), (7.0, 8.0));
            line((7.0, 3.0), (7.0, 8.0));
        }
        Icon::Close => {
            line((-4.5, -4.5), (4.5, 4.5));
            line((4.5, -4.5), (-4.5, 4.5));
        }
        Icon::Display => {
            box_at(0.0, -1.5, 17.0, 12.0);
            line((0.0, 5.0), (0.0, 8.0));
            line((-4.0, 8.0), (4.0, 8.0));
        }
        Icon::Palette => {
            p.circle_stroke(c, 8.0 * scale, stroke);
            for (x, y) in [(-3.5, -3.0), (2.0_f32, -4.0), (4.0, 1.5), (-3.0, 3.0)] {
                p.circle_filled(point(x, y), 1.5 * scale, color);
            }
        }
        Icon::AppLocation => {
            box_at(0.0, 0.0, 16.0, 14.0);
            line((-6.0, -3.0), (6.0, -3.0));
            line((-3.0, 2.0), (3.0, 2.0));
            line((1.0, 0.0), (3.0, 2.0));
            line((3.0, 2.0), (1.0, 4.0));
        }
    }
}

pub fn rect_rotated(
    p: &Painter,
    rect: Rect,
    radius: f32,
    angle: f32,
    fill: Color32,
    stroke: Stroke,
) {
    let inset = (stroke.width * 0.5)
        .max(0.0)
        .min(rect.width().min(rect.height()) * 0.45);
    let rect = rect.shrink(inset);
    let radius = radius.max(0.0).min(rect.width().min(rect.height()) * 0.5);
    let center = rect.center();
    let (sin, cos) = angle.sin_cos();
    let rotate = |point: egui::Pos2| {
        let delta = point - center;
        center + Vec2::new(delta.x * cos - delta.y * sin, delta.x * sin + delta.y * cos)
    };
    let corners = [
        (
            egui::pos2(rect.left() + radius, rect.top() + radius),
            std::f32::consts::PI,
        ),
        (
            egui::pos2(rect.right() - radius, rect.top() + radius),
            std::f32::consts::PI * 1.5,
        ),
        (
            egui::pos2(rect.right() - radius, rect.bottom() - radius),
            0.0,
        ),
        (
            egui::pos2(rect.left() + radius, rect.bottom() - radius),
            std::f32::consts::FRAC_PI_2,
        ),
    ];
    let mut points = Vec::with_capacity(20);
    for (corner, start) in corners {
        for step in 0..=4 {
            let a = start + std::f32::consts::FRAC_PI_2 * step as f32 / 4.0;
            points.push(rotate(corner + Vec2::angled(a) * radius));
        }
    }
    p.add(egui::Shape::convex_polygon(points, fill, stroke));
}

/// One continuous circular arrow. The arrow tip is exactly the end of the arc.
pub fn paint_refresh(p: &Painter, rect: Rect, color: Color32, angle: f32) {
    let scale = rect.width().min(rect.height()) / 20.0;
    let c = rect.center();
    let radius = 6.0 * scale;
    let start = -0.40;
    let end = std::f32::consts::TAU - 1.10;
    let points = (0..=32)
        .map(|i| c + Vec2::angled(start + (end - start) * i as f32 / 32.0 + angle) * radius)
        .collect::<Vec<_>>();
    let tip = *points.last().unwrap();
    let tangent = Vec2::angled(end + angle + std::f32::consts::FRAC_PI_2);
    let normal = Vec2::angled(end + angle);
    let stroke = Stroke::new(1.5 * scale, color);
    p.add(egui::Shape::line(points, stroke));
    p.add(egui::Shape::line(
        vec![
            tip - tangent * 2.8 * scale - normal * 1.8 * scale,
            tip,
            tip - tangent * 2.8 * scale + normal * 1.8 * scale,
        ],
        stroke,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;
    #[test]
    fn refresh_arc_and_arrow_join_and_remain_bounded_through_a_full_turn() {
        let ctx = egui::Context::default();
        let slot = Rect::from_min_size(Pos2::new(20.0, 20.0), Vec2::splat(20.0));
        for i in 0..36 {
            let out = ctx.run(egui::RawInput::default(), |ctx| {
                paint_refresh(
                    &ctx.layer_painter(egui::LayerId::background()),
                    slot,
                    Color32::WHITE,
                    i as f32 * std::f32::consts::TAU / 36.0,
                )
            });
            let paths = out
                .shapes
                .iter()
                .filter_map(|s| {
                    if let egui::Shape::Path(p) = &s.shape {
                        Some(p)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(paths.len(), 2);
            assert_eq!(paths[0].points.last(), Some(&paths[1].points[1]));
            for shape in out.shapes {
                assert!(slot
                    .expand(0.1)
                    .contains_rect(shape.shape.visual_bounding_rect()));
            }
        }
    }
    #[test]
    fn every_vector_icon_stroke_stays_inside_its_slot() {
        let ctx = egui::Context::default();
        for icon in [
            Icon::Library,
            Icon::Script,
            Icon::Plugin,
            Icon::Folder,
            Icon::Reveal,
            Icon::Info,
            Icon::Settings,
            Icon::Search,
            Icon::Refresh,
            Icon::Trash,
            Icon::Import,
            Icon::Close,
            Icon::Display,
            Icon::Palette,
            Icon::AppLocation,
        ] {
            for size in [16.0, 18.0, 20.0, 28.0] {
                let slot = Rect::from_min_size(Pos2::new(40.0, 40.0), Vec2::splat(size));
                let output = ctx.run(egui::RawInput::default(), |ctx| {
                    paint(
                        &ctx.layer_painter(egui::LayerId::new(
                            egui::Order::Foreground,
                            egui::Id::new("icon-test"),
                        )),
                        slot,
                        icon,
                        Color32::WHITE,
                    );
                });
                for shape in output.shapes {
                    assert!(
                        slot.expand(0.1)
                            .contains_rect(shape.shape.visual_bounding_rect()),
                        "{icon:?} {size}: {:?}",
                        shape.shape.visual_bounding_rect()
                    );
                }
            }
        }
    }
    #[test]
    fn rotated_cards_and_icons_stay_inside_their_audit_bounds() {
        let ctx = egui::Context::default();
        for angle in [-0.14, -0.10, 0.08, 0.12] {
            let slot = Rect::from_min_size(Pos2::new(40.0, 40.0), Vec2::new(76.0, 56.0));
            let output = ctx.run(egui::RawInput::default(), |ctx| {
                let painter = ctx.layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("rotated-icon-test"),
                ));
                rect_rotated(
                    &painter,
                    slot,
                    12.0,
                    angle,
                    Color32::TRANSPARENT,
                    Stroke::new(0.5_f32, Color32::WHITE),
                );
                paint_rotated(
                    &painter,
                    Rect::from_center_size(slot.center(), Vec2::splat(24.0)),
                    Icon::Script,
                    Color32::WHITE,
                    angle,
                );
            });
            for shape in output.shapes {
                assert!(
                    slot.expand(4.1)
                        .contains_rect(shape.shape.visual_bounding_rect()),
                    "{angle}: {:?}",
                    shape.shape.visual_bounding_rect()
                );
            }
        }
    }
}
