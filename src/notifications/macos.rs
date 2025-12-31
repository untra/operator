use anyhow::Result;
use mac_notification_sys::{Notification, NotificationResponse};

pub fn send_notification(title: &str, subtitle: &str, message: &str, sound: bool) -> Result<()> {
    let mut notification = Notification::new();

    notification
        .title(title)
        .subtitle(subtitle)
        .message(message);

    if sound {
        notification.sound("default");
    }

    // We don't need to handle the response for now
    let _ = notification.send();

    Ok(())
}

/// Send a notification that can be clicked to perform an action
#[allow(dead_code)]
pub fn send_actionable(
    title: &str,
    subtitle: &str,
    message: &str,
    action_label: &str,
    sound: bool,
) -> Result<bool> {
    let mut notification = Notification::new();

    notification
        .title(title)
        .subtitle(subtitle)
        .message(message)
        .main_button(mac_notification_sys::MainButton::SingleAction(action_label));

    if sound {
        notification.sound("default");
    }

    match notification.send() {
        Ok(NotificationResponse::ActionButton(_)) => Ok(true),
        Ok(_) => Ok(false),
        Err(e) => {
            tracing::warn!("Notification error: {:?}", e);
            Ok(false)
        }
    }
}
