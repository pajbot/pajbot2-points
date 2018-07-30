/*
pub fn buf_to_u32(buffer: [u8; 4]) -> u32 {
    return ((buffer[0] as u32) << 24)
        + ((buffer[1] as u32) << 16)
        + ((buffer[2] as u32) << 8)
        + ((buffer[3] as u32));
}
*/

pub fn buf_to_i32_unsafe(buffer: &[u8]) -> i32 {
    return ((buffer[0] as i32) << 24)
        + ((buffer[1] as i32) << 16)
        + ((buffer[2] as i32) << 8)
        + ((buffer[3] as i32));
}

pub fn u64_to_buf(value: u64) -> [u8; 8] {
    let mut buffer = [0; 8];
    buffer[0] = ((value >> 56) & 0xFF) as u8;
    buffer[1] = ((value >> 48) & 0xFF) as u8;
    buffer[2] = ((value >> 40) & 0xFF) as u8;
    buffer[3] = ((value >> 32) & 0xFF) as u8;
    buffer[4] = ((value >> 24) & 0xFF) as u8;
    buffer[5] = ((value >> 16) & 0xFF) as u8;
    buffer[6] = ((value >> 8) & 0xFF) as u8;
    buffer[7] = ((value) & 0xFF) as u8;

    return buffer;
}

pub fn buf_to_u32_unsafe(buffer: &[u8]) -> u32 {
    return ((buffer[0] as u32) << 24)
        + ((buffer[1] as u32) << 16)
        + ((buffer[2] as u32) << 8)
        + ((buffer[3] as u32));
}
