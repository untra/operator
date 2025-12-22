use anyhow::Result;
use notify_rust::Notification;

pub fn send_notification(title: &str, subtitle: &str, message: &str, _sound: bool) -> Result<()> {
    // Combine subtitle and message for freedesktop format
    let body = if subtitle.is_empty() {
        message.to_string()
    } else {
        format!("{}\n{}", subtitle, message)
    };

    Notification::new()
        .summary(title)
        .body(&body)
        .appname("operator")
        .show()?;

    Ok(())
}
