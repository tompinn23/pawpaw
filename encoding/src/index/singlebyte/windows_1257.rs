// AUTOGENERATED FROM index-windows-1257.txt, ORIGINAL COMMENT FOLLOWS:
//
// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/
//
// For details on index index-windows-1257.txt see the Encoding Standard
// https://encoding.spec.whatwg.org/
//
// Identifier: cc7256bdd10a5b8dc7fb6f994659f307dfcae60def9aa6c29d811f85e2842c47
// Date: 2016-01-20

#[allow(dead_code)]
const X: u16 = 0xffff;

const FORWARD_TABLE: &'static [u16] = &[
    8364, 129, 8218, 131, 8222, 8230, 8224, 8225, 136, 8240, 138, 8249, 140, 168, 711, 184, 144,
    8216, 8217, 8220, 8221, 8226, 8211, 8212, 152, 8482, 154, 8250, 156, 175, 731, 159, 160, X,
    162, 163, 164, X, 166, 167, 216, 169, 342, 171, 172, 173, 174, 198, 176, 177, 178, 179, 180,
    181, 182, 183, 248, 185, 343, 187, 188, 189, 190, 230, 260, 302, 256, 262, 196, 197, 280, 274,
    268, 201, 377, 278, 290, 310, 298, 315, 352, 323, 325, 211, 332, 213, 214, 215, 370, 321, 346,
    362, 220, 379, 381, 223, 261, 303, 257, 263, 228, 229, 281, 275, 269, 233, 378, 279, 291, 311,
    299, 316, 353, 324, 326, 243, 333, 245, 246, 247, 371, 322, 347, 363, 252, 380, 382, 729,
]; // 128 entries

/// Returns the index code point for pointer `code` in this index.
#[inline]
pub fn forward(code: u8) -> u16 {
    FORWARD_TABLE[(code - 0x80) as usize]
}

#[cfg(not(feature = "no-optimized-legacy-encoding"))]
const BACKWARD_TABLE_LOWER: &'static [u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    194, 226, 0, 0, 192, 224, 195, 227, 0, 0, 0, 0, 200, 232, 0, 0, 0, 0, 199, 231, 0, 0, 203, 235,
    198, 230, 0, 0, 0, 0, 0, 0, 0, 0, 204, 236, 0, 0, 0, 0, 0, 0, 206, 238, 0, 0, 193, 225, 0, 0,
    0, 0, 0, 0, 205, 237, 0, 0, 0, 207, 239, 0, 0, 0, 0, 0, 0, 0, 142, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 158, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 153, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 150, 151,
    0, 0, 0, 145, 146, 130, 0, 147, 148, 132, 0, 134, 135, 149, 0, 0, 0, 133, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 137, 0, 0, 0, 0, 0, 0, 0, 0, 139, 155, 0, 0, 0, 0, 0, 196, 197, 175, 0, 0, 201, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 211, 0, 213, 214, 215, 168, 0, 0, 0, 220, 0, 0, 223, 0, 0, 0, 0, 228, 229,
    191, 0, 0, 233, 0, 0, 0, 0, 0, 0, 0, 0, 0, 243, 0, 245, 246, 247, 184, 0, 0, 0, 252, 0, 0, 0,
    129, 0, 131, 0, 0, 0, 0, 136, 0, 138, 0, 140, 0, 0, 0, 144, 0, 0, 0, 0, 0, 0, 0, 152, 0, 154,
    0, 156, 0, 0, 159, 160, 0, 162, 163, 164, 0, 166, 167, 141, 169, 0, 171, 172, 173, 174, 157,
    176, 177, 178, 179, 180, 181, 182, 183, 143, 185, 0, 187, 188, 189, 190, 0, 217, 249, 209, 241,
    210, 242, 0, 0, 0, 0, 0, 212, 244, 0, 0, 0, 0, 0, 0, 0, 0, 170, 186, 0, 0, 218, 250, 0, 0, 0,
    0, 208, 240, 0, 0, 0, 0, 0, 0, 0, 0, 219, 251, 0, 0, 0, 0, 0, 0, 216, 248, 0, 0, 0, 0, 0, 202,
    234, 221, 253, 222, 254, 0,
]; // 493 entries

#[cfg(not(feature = "no-optimized-legacy-encoding"))]
const BACKWARD_TABLE_UPPER: &'static [u16] = &[
    0, 0, 366, 303, 64, 429, 0, 0, 0, 0, 0, 125, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 243, 0, 153, 0, 198,
]; // 133 entries

/// Returns the index pointer for code point `code` in this index.
#[inline]
#[cfg(not(feature = "no-optimized-legacy-encoding"))]
pub fn backward(code: u32) -> u8 {
    let offset = (code >> 6) as usize;
    let offset = if offset < 133 {
        BACKWARD_TABLE_UPPER[offset] as usize
    } else {
        0
    };
    BACKWARD_TABLE_LOWER[offset + ((code & 63) as usize)]
}

/// Returns the index pointer for code point `code` in this index.
#[cfg(feature = "no-optimized-legacy-encoding")]
pub fn backward(code: u32) -> u8 {
    if code > 8482 || ((0x10003u32 >> (code >> 9)) & 1) == 0 {
        return 0;
    }
    let code = code as u16;
    for i in 0..0x80 {
        if FORWARD_TABLE[i as usize] == code {
            return 0x80 + i;
        }
    }
    0
}

#[cfg(test)]
single_byte_tests! {}
