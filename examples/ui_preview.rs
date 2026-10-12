//! Render actual egui UI offscreen with synthetic data. No window control or Cavalry operations.
use cavalrykit::{settings::Language, ui::app::CavalryKitApp};
use eframe::{
    egui_wgpu::{Renderer, ScreenDescriptor},
    wgpu,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "private/ui-preview".into());
    std::fs::create_dir_all(&output)?;
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        ..Default::default()
    }))
    .ok_or("No offscreen GPU adapter")?;
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))?;
    for (language, screen) in [
        (Language::Zh, "main"),
        (Language::Zh, "light-main"),
        (Language::Zh, "light-appearance"),
        (Language::En, "light-settings"),
        (Language::Zh, "light-main-small"),
        (Language::En, "light-main-small"),
        (Language::Zh, "light-result"),
        (Language::Zh, "settings"),
        (Language::Zh, "appearance"),
        (Language::Zh, "installation"),
        (Language::Zh, "cavalry-settings"),
        (Language::Zh, "result"),
        (Language::Zh, "conflict"),
        (Language::Zh, "restart"),
        (Language::Zh, "custom"),
        (Language::En, "settings"),
        (Language::Ja, "settings"),
        (Language::Ko, "settings"),
        (Language::En, "light"),
        (Language::Zh, "main-small"),
        (Language::En, "main-small"),
        (Language::Ja, "main-small"),
        (Language::Ko, "main-small"),
        (Language::Zh, "custom-small"),
        (Language::Ja, "settings-small"),
        (Language::En, "appearance-small"),
        (Language::Ko, "installation-small"),
    ] {
        let compact = screen.ends_with("-small");
        let (width, height) = if compact {
            (720u32, 496u32)
        } else {
            (1120u32, 760u32)
        };
        let ctx = egui::Context::default();
        let mut app = CavalryKitApp::preview(
            &ctx,
            language,
            screen.strip_suffix("-small").unwrap_or(screen),
        );
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let mut renderer = Renderer::new(&device, format, None, 1, false);
        let mut final_frame = None;
        for frame in 0..6 {
            let result = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width as f32, height as f32),
                    )),
                    time: Some(frame as f64),
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
            for (id, delta) in &result.textures_delta.set {
                renderer.update_texture(&device, &queue, *id, delta);
            }
            final_frame = Some(result);
        }
        let result = final_frame.unwrap();
        let jobs = ctx.tessellate(result.shapes, result.pixels_per_point);
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("CavalryKit UI preview"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: result.pixels_per_point,
        };
        let mut encoder = device.create_command_encoder(&Default::default());
        let command_buffers =
            renderer.update_buffers(&device, &queue, &mut encoder, &jobs, &descriptor);
        {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            renderer.render(&mut pass.forget_lifetime(), &jobs, &descriptor);
        }
        let padded = (width * 4).div_ceil(256) * 256;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (padded * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(
            command_buffers
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        device.poll(wgpu::Maintain::Wait);
        rx.recv()??;
        let bytes = buffer.slice(..).get_mapped_range();
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for row in bytes.chunks(padded as usize) {
            rgba.extend_from_slice(&row[..(width * 4) as usize]);
        }
        let image =
            image::RgbaImage::from_raw(width, height, rgba).ok_or("Invalid image dimensions")?;
        let name = format!("{output}/{language:?}-{screen}.png");
        image.save(&name)?;
        println!("{name}");
        drop(bytes);
        buffer.unmap();
    }
    Ok(())
}
