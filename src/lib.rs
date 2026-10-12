//! CavalryKit 库入口：引擎模块 + UI 模块 + 应用启动函数。

pub mod detector;
pub mod installer;
pub mod library;
pub mod paths;
pub mod ui;
pub mod unzipper;
pub mod worker;

use std::path::PathBuf;

/// 应用启动（原 main 逻辑，便于二进制入口与测试复用）。
pub fn run() -> eframe::Result<()> {
    // Log to stderr only when requested; never create logs containing imported private paths.
    let _ = env_logger::try_init();
    paths::setup_app_identity();

    // 通过「打开方式」/命令行传入的文件路径，进入初始安装队列
    let initial: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(concat!("CavalryKit ", env!("CARGO_PKG_VERSION")))
            .with_app_id("com.cavalrykit.app")
            .with_inner_size([960.0, 680.0])
            .with_min_inner_size([800.0, 560.0])
            .with_decorations(true)
            .with_transparent(false)
            .with_icon(ui::icon::window_icon()),
        // Use native Metal / DX12/Vulkan rather than relying on Windows OpenGL drivers.
        // Keep an explicit compatibility escape hatch for older graphics drivers.
        renderer: if std::env::var("CAVALRYKIT_RENDERER").as_deref() == Ok("glow") {
            eframe::Renderer::Glow
        } else {
            eframe::Renderer::Wgpu
        },
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            desired_maximum_frame_latency: Some(1),
            ..Default::default()
        },
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "CavalryKit",
        options,
        Box::new(move |cc| {
            // Reset UI zoom; physical scaling remains managed by the native per-monitor DPI.
            cc.egui_ctx.set_zoom_factor(1.0);
            ui::fonts::setup(&cc.egui_ctx);
            ui::theme::apply(&cc.egui_ctx);
            Ok(Box::new(ui::app::CavalryKitApp::new(&cc.egui_ctx, initial)))
        }),
    )
}
