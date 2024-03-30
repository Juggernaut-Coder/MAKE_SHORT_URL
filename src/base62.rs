const BASE: u64 = 62;
const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub async fn encode(mut num: u64) -> String {
    if num == 0 {
        return "0".into();
    }
    let mut result_str = String::new();
    while num > 0 {
        let remainder = (num % BASE) as usize;
        num /= BASE;
        result_str.push(ALPHABET[remainder] as char);
    }
    return result_str.chars().rev().collect();
}

pub async fn decode(input: &str) -> u64 {
    let mut num = 0u64;
    for char in input.chars() {
        let index = ALPHABET.iter().position(|&c| c as char == char).unwrap();
        num = num * BASE + index as u64;
    }
    return num;
}