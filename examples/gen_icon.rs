//! 生成应用图标 assets/icon.png（1024x1024）：
//! MD3 品牌:主紫 #6750A4 squircle + 白色 ">_" 终端字形(与应用内 logo 块一致)。
//! 2048 超采样绘制后 Lanczos 降到 1024,边缘平滑。
//! 运行：cargo run --example gen_icon

use image::{Rgba, RgbaImage};

const DRAW: u32 = 2048; // 超采样画布
const OUT: u32 = 1024;

const PURPLE: (u8, u8, u8) = (0x67, 0x50, 0xa4); // MD3 primary
const WHITE: (u8, u8, u8) = (0xff, 0xff, 0xff);

fn squircle_mask(px: f32, py: f32, size: f32, radius: f32) -> bool {
    // 圆角矩形包含测试(含边框 0.5px 软化)
    let r = radius;
    let cx = px.clamp(r, size - r);
    let cy = py.clamp(r, size - r);
    let dx = px - cx;
    let dy = py - cy;
    dx * dx + dy * dy <= r * r
}

/// 点到线段距离的平方
fn seg_dist2(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let vx = bx - ax;
    let vy = by - ay;
    let wx = px - ax;
    let wy = py - ay;
    let len2 = vx * vx + vy * vy;
    let t = if len2 > 0.0 {
        (wx * vx + wy * vy) / len2
    } else {
        0.0
    }
    .clamp(0.0, 1.0);
    let dx = px - (ax + vx * t);
    let dy = py - (ay + vy * t);
    dx * dx + dy * dy
}

fn main() {
    let size = DRAW as f32;
    let radius = size * 0.223; // macOS 图标圆角率
    let mut img = RgbaImage::from_pixel(DRAW, DRAW, Rgba([0, 0, 0, 0]));

    // 「>_」字形几何(居中,以画布比例定义)
    let c = size / 2.0;
    let stroke = size * 0.062; // 笔画宽
    let rr = (stroke / 2.0) * (stroke / 2.0);
    // 尖括号 ">":两条斜线
    let ch_top = c - size * 0.135;
    let ch_mid_y = c - size * 0.01;
    let ch_bot = c + size * 0.115;
    let ch_left = c - size * 0.20;
    let ch_right = c - size * 0.02;
    // 下划线 "_"
    let us_x0 = c + size * 0.06;
    let us_x1 = c + size * 0.24;
    let us_y = c + size * 0.115;

    for y in 0..DRAW {
        for x in 0..DRAW {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            if !squircle_mask(px, py, size, radius) {
                continue;
            }
            // 紫底
            let mut col = Rgba([PURPLE.0, PURPLE.1, PURPLE.2, 255]);
            // 白色字形(圆头笔画)
            let d_chev = seg_dist2(px, py, ch_left, ch_top, ch_right, ch_mid_y)
                .min(seg_dist2(px, py, ch_right, ch_mid_y, ch_left, ch_bot));
            let d_us = seg_dist2(px, py, us_x0, us_y, us_x1, us_y);
            if d_chev <= rr || d_us <= rr {
                col = Rgba([WHITE.0, WHITE.1, WHITE.2, 255]);
            }
            img.put_pixel(x, y, col);
        }
    }

    let small = image::imageops::resize(
        &img,
        OUT,
        OUT,
        image::imageops::FilterType::Lanczos3,
    );
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/icon.png");
    small.save(&out).expect("保存 icon.png 失败");
    println!("已生成 {}", out.display());
}
