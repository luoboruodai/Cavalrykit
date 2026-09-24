//! CavalryKit 库入口：引擎模块 + UI 模块 + 应用启动函数。

pub mod detector;
pub mod installer;
pub mod library;
pub mod paths;
pub mod ui;
pub mod unzipper;

use std::path::PathBuf;

/// 应用启动（原 main 逻辑，便于二进制入口与测试复用）。
pub fn run() -> eframe::Result<()> {
    // 日志默认写到系统临时目录的 cavalrykit.log，便于排查启动问题
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    if std::env::var("RUST_LOG_STYLE").is_err() {
        std::env::set_var("RUST_LOG_STYLE", "always");
    }
    env_logger::init();

    // 通过「打开方式」/命令行传入的文件路径，进入初始安装队列
    let initial: Vec<PathBuf> = std::env::args()
        .skip(1)
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect();

    // 验证用:CAVALRYKIT_VERIFY_SIZE="WxH" 覆盖窗口尺寸并隐藏标题栏;
    // 内容按 w/880 等比 zoom,让 1536 捕获与 comp 同尺度(impeccable hero 门在 comp 像素上对比)
    let (w, h, verify) = std::env::var("CAVALRYKIT_VERIFY_SIZE")
        .ok()
        .and_then(|s| {
            let (a, b) = s.split_once('x')?;
            Some((a.parse::<f32>().ok()?, b.parse::<f32>().ok()?))
        })
        .map(|(a, b)| (a, b, true))
        .unwrap_or((880.0, 660.0, false));
    let verify_zoom = if verify { w / 880.0 } else { 1.0 };

    let mut vp = egui::ViewportBuilder::default()
        .with_title("CavalryKit")
        .with_titlebar_shown(!verify)
        .with_inner_size([w, h])
        .with_min_inner_size([740.0, 540.0])
        .with_icon(ui::icon::window_icon());
    if verify {
        // 验证时固定到主屏左上角(多显示器下默认落点可能在副屏顶部被系统关掉)
        vp = vp.with_position([1560.0, 28.0]);
    }

    let options = eframe::NativeOptions {
        viewport: vp,
        ..Default::default()
    };

    eframe::run_native(
        "CavalryKit",
        options,
        Box::new(move |cc| {
            // 验证缩放仅验证时生效;正常启动显式复位,避免持久化泄漏(egui memory 会记住 zoom)
            cc.egui_ctx.set_zoom_factor(verify_zoom);
            ui::fonts::setup(&cc.egui_ctx);
            ui::theme::apply(&cc.egui_ctx);
            Ok(Box::new(ui::app::CavalryKitApp::new(initial)))
        }),
    )
}
