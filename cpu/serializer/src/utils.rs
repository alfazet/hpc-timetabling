pub(crate) fn bit_string_u8(value: u8, len: u32) -> String {
    let mut s = String::with_capacity(len as usize);
    for i in 0..len {
        s.push(if (value & (1 << i)) != 0 { '1' } else { '0' });
    }
    s
}

pub(crate) fn bit_string_u16(value: u16, len: u32) -> String {
    let mut s = String::with_capacity(len as usize);
    for i in 0..len {
        s.push(if (value & (1 << i)) != 0 { '1' } else { '0' });
    }
    s
}
