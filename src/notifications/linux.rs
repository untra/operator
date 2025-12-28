use anyhow::Result;
use notify_rust::Notification;

pub fn send_notification(title: &str, subtitle: &str, message: &str, _sound: bool) -> Result<()> {
    // Combine subtitle and message for freedesktop format
    let body = if subtitle.is_empty() {
        message.to_string()
    } else {
        format!("{}\n{}", subtitle, message)
    };

    // Handle D-Bus errors gracefully - notification daemon may not be available
    // in CI environments or headless systems
    match Notification::new()
        .summary(title)
        .body(&body)
        .appname("operator")
        .show()
    {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::warn!("Failed to send notification (D-Bus unavailable?): {}", e);
            Ok(())
        }
    }
}
