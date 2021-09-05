use rand::{distributions::Alphanumeric, Rng};

pub fn is_string_server_valid(s: &str) -> bool {
    if s.trim().is_empty() {
        return false;
    }
    is_optional_string_server_valid(s)
}

pub fn is_optional_string_server_valid(s: &str) -> bool {
    s.chars().all(|c| c.is_alphanumeric() || c == '@' || c == '.' || c == '-' || c == '_' || c == '!' || c == ' ')
}

pub fn is_email_valid(s: &str) -> bool {
    if is_string_server_valid(s) {
        if let Some((user, host)) = s.split_once('@') {
            if user.len() > 0 && host.len() > 0 && !host.contains('@') && host.contains('.') {
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

// Note: passwords dont have to abide by the char restrictions of most other inputs since they are never shown.
pub fn is_password_valid(p: &str) -> bool {
    if p.len() >= 6 {
        true
    } else {
        false
    }
}

pub fn gen_password(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}