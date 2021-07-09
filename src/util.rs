use rand::{distributions::Alphanumeric, Rng};

pub fn is_string_server_valid(s: &str) -> bool {
    if s.trim().is_empty() {
        return false;
    }
    s.chars().all(|c| c.is_alphanumeric() || c == '@' || c == '.' || c == '-' || c == '_' || c == '!' || c == ' ')
}

pub fn gen_password(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}