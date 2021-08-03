use crate::data::SharedData;
use async_std::task;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;

pub async fn user_notification_process(data: Arc<SharedData>) {
    println!("Starting user notification process...");
    loop {
        let now = SystemTime::now();
        data.org_db.for_each_write(|mut org| {
            if now > org.last_notification + org.notification_interval {
                
                // Send notification
                let unreviewed_count = org.unreviewed_sections.len();

                let mut send_count = 0;

                for user_id in org.associates.iter() {
                    if let Ok(Some(user)) = data.user_db.fetch(user_id) {
                        if user.notifications && unreviewed_count > 0 {
                            // Send email
                            if let Err(e) = data.send_email(
                                &user.email,
                                &format!("There are {} new unreviewed sections", unreviewed_count),
                                "Unreviewed Sections",
                                &format!("There are {} new unreviewed sections", unreviewed_count),
                                "Sign in to your account to view these unread sections.",
                            ) {
                                println!("Failed to send notification email: {}", e);
                            } else {
                                send_count += 1;
                            }
                        }
                    }
                }

                println!("Sent {} notifications to org: {}", send_count, &org.name);

                org.last_notification = now;
            }
            // Org is written back if changed due to WriteGuard.
        });
        // Long sleep before next tick - we dont need precise timing on this.
        task::sleep(Duration::from_secs(500)).await;
    }
}
