//! Small, consistent vector icons. Never depend on symbol-font coverage.
use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};
#[derive(Clone, Copy)]
pub enum Icon {
    Library,
    Script,
    Plugin,
    Folder,
    Info,
    Settings,
    Search,
    Refresh,
    Trash,
    Import,
    Close,
    Display,
    Palette,
    Cavalry,
}
pub fn paint(p: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let c = rect.center();
    let scale = rect.width().min(rect.height()) / 20.0;
    let point = |x, y| c + Vec2::new(x, y) * scale;
    let stroke = Stroke::new(1.4 * scale, color);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([point(a.0, a.1), point(b.0, b.1)], stroke);
    };
    let box_at = |x, y, w, h| {
        p.rect_stroke(
            Rect::from_center_size(point(x, y), Vec2::new(w, h) * scale),
            2,
            stroke,
            egui::StrokeKind::Inside,
        );
    };
    match icon {
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
            box_at(0.0, 1.0_f32, 13.0, 12.0);
            p.circle_stroke(point(0.0, -6.0), 2.5 * scale, stroke);
            p.circle_stroke(point(7.0, 1.0_f32), 2.5 * scale, stroke);
        }
        Icon::Folder => {
            box_at(0.0, 1.0_f32, 16.0, 12.0);
            line((-7.0, -6.0), (-2.0_f32, -6.0));
            line((-2.0_f32, -6.0), (0.0, -4.0));
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
        Icon::Refresh => {
            let pts: Vec<Pos2> = (0..=22)
                .map(|i| {
                    let a = 0.5 + i as f32 * 5.0 / 22.0;
                    point(a.cos() * 6.5, a.sin() * 6.5)
                })
                .collect();
            p.add(egui::Shape::line(pts, stroke));
            line((5.0, -6.0), (5.0, -1.0_f32));
            line((5.0, -1.0_f32), (0.0, -1.0_f32));
        }
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
        Icon::Cavalry => {
            line((-7.0, 7.0), (0.0, -7.0));
            line((0.0, -7.0), (7.0, 7.0));
            line((-3.5, 1.0_f32), (3.5, 1.0_f32));
        }
    }
}
