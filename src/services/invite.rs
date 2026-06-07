use rand::{distributions::Alphanumeric, Rng};

pub fn generate_invite_code() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect()
}
