use anyhow::{Context, Result};
use tauri::{PhysicalPosition, PhysicalSize, Window};

pub fn configure_window(window: &Window) -> Result<()> {
    if let Some(monitor) = window.current_monitor()? {
        let size = monitor.size();
        let width = size.width as f32;
        let height = size.height as f32;

        window
            .set_size(PhysicalSize::new(width * 0.8, height * 0.8))
            .context("Failed to set window size")?;

        window
            .set_position(PhysicalPosition::new(width * 0.1, height * 0.1))
            .context("Failed to set window position")?;
    }
    Ok(())
}
