const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode(data: &[u8]) -> String {
    let mut o = String::with_capacity(data.len().div_ceil(3) * 4);
    for ch in data.chunks(3) {
        let b0 = u32::from(ch[0]);
        let b1 = u32::from(*ch.get(1).unwrap_or(&0));
        let b2 = u32::from(*ch.get(2).unwrap_or(&0));
        let num = b0 << 16 | b1 << 8 | b2;

        o.push(ALPHABET[(num >> 18) as usize & 0x3f] as char);

        o.push(ALPHABET[(num >> 12) as usize & 0x3f] as char);

        o.push(if ch.len() > 1 {
            ALPHABET[(num >> 6) as usize & 0x3f] as char
        } else {
            '='
        });

        o.push(if ch.len() > 2 {
            ALPHABET[num as usize & 0x3f] as char
        } else {
            '='
        });
    }

    o
}
