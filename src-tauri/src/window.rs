#[cfg(target_os = "macos")]
pub(crate) mod macos;

use tauri::{PhysicalPosition, PhysicalSize, Window};

pub fn configure_window(window: &Window) {
    match window.current_monitor() {
        Ok(Some(monitor)) => {
            let size = monitor.size();
            let width = size.width as f32;
            let height = size.height as f32;

            window
                .set_size(PhysicalSize::new(width * 0.8, height * 0.8))
                .expect("Couldn't resize window");

            window
                .set_position(PhysicalPosition::new(width * 0.1, height * 0.1))
                .expect("Couldn't position window");
        }
        Ok(None) => {
            log::warn!("No monitor found, cannot configure window size and position.");
        }
        Err(e) => {
            log::error!("Failed to get current monitor: {}", e);
        }
    }
}
