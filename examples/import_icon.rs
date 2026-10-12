//! Convert the checked-in ICO to the PNG used by the window and macOS bundle.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let icon = image::open("assets/icon.ico")?;
    icon.to_rgba8().save("assets/icon.png")?;
    println!(
        "Updated assets/icon.png ({}x{})",
        icon.width(),
        icon.height()
    );
    Ok(())
}
