use rand::Rng;
use std::iter;

static CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvxyz";
static TOKEN_LENGTH: usize = 6;

pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let get_one_char = || CHARSET[rng.gen_range(0..CHARSET.len())] as char;
    iter::repeat_with(get_one_char).take(TOKEN_LENGTH).collect()
}
