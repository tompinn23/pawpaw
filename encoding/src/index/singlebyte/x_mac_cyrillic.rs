// AUTOGENERATED FROM index-x-mac-cyrillic.txt, ORIGINAL COMMENT FOLLOWS:
//
// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/
//
// For details on index index-x-mac-cyrillic.txt see the Encoding Standard
// https://encoding.spec.whatwg.org/
//
// Identifier: 73e8e7642c6fa9de29d42819b47fba55b58666fb1e339faeb4a89a0bd7c24d43
// Date: 2016-01-20

#[allow(dead_code)]
const X: u16 = 0xffff;

const FORWARD_TABLE: &'static [u16] = &[
    1040, 1041, 1042, 1043, 1044, 1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052, 1053, 1054, 1055,
    1056, 1057, 1058, 1059, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067, 1068, 1069, 1070, 1071,
    8224, 176, 1168, 163, 167, 8226, 182, 1030, 174, 169, 8482, 1026, 1106, 8800, 1027, 1107, 8734,
    177, 8804, 8805, 1110, 181, 1169, 1032, 1028, 1108, 1031, 1111, 1033, 1113, 1034, 1114, 1112,
    1029, 172, 8730, 402, 8776, 8710, 171, 187, 8230, 160, 1035, 1115, 1036, 1116, 1109, 8211,
    8212, 8220, 8221, 8216, 8217, 247, 8222, 1038, 1118, 1039, 1119, 8470, 1025, 1105, 1103, 1072,
    1073, 1074, 1075, 1076, 1077, 1078, 1079, 1080, 1081, 1082, 1083, 1084, 1085, 1086, 1087, 1088,
    1089, 1090, 1091, 1092, 1093, 1094, 1095, 1096, 1097, 1098, 1099, 1100, 1101, 1102, 8364,
]; // 128 entries

/// Returns the index code point for pointer `code` in this index.
#[inline]
pub fn forward(code: u8) -> u16 {
    FORWARD_TABLE[(code - 0x80) as usize]
}

#[cfg(not(feature = "no-optimized-legacy-encoding"))]
const BACKWARD_TABLE_LOWER: &'static [u8] = &[
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    173, 0, 0, 0, 178, 179, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 220, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 162, 182, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 197, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 196, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 198,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 195, 0, 0, 0, 176, 0, 144, 145, 146,
    147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 224, 225, 226, 227, 228, 229,
    230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 160, 0, 165, 0, 0, 0, 201, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 208, 209, 0, 0, 0, 212, 213, 0, 0,
    210, 211, 215, 0, 202, 0, 0, 163, 0, 0, 0, 164, 0, 169, 0, 199, 194, 0, 168, 0, 161, 177, 0, 0,
    0, 181, 166, 0, 0, 0, 0, 200, 0, 0, 0, 0, 170, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 214, 0, 0, 0, 0, 0, 0, 0, 0, 221, 171, 174, 184, 193,
    167, 186, 183, 188, 190, 203, 205, 0, 216, 218, 128, 129, 130, 131, 132, 133, 134, 135, 136,
    137, 138, 139, 140, 141, 142, 143, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251,
    252, 253, 254, 223, 0, 222, 172, 175, 185, 207, 180, 187, 192, 189, 191, 204, 206, 0, 217, 219,
]; // 388 entries

#[cfg(not(feature = "no-optimized-legacy-encoding"))]
const BACKWARD_TABLE_UPPER: &'static [u16] = &[
    0, 0, 0, 0, 0, 254, 0, 293, 0, 0, 0, 0, 119, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 324, 177, 356, 0, 81, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 222, 209, 0, 0, 0, 65, 0, 0, 42, 284, 0, 0, 0, 0, 0, 0, 145, 0, 105, 32,
]; // 276 entries

/// Returns the index pointer for code point `code` in this index.
#[inline]
#[cfg(not(feature = "no-optimized-legacy-encoding"))]
pub fn backward(code: u32) -> u8 {
    let offset = (code >> 5) as usize;
    let offset = if offset < 276 {
        BACKWARD_TABLE_UPPER[offset] as usize
    } else {
        0
    };
    BACKWARD_TABLE_LOWER[offset + ((code & 31) as usize)]
}

/// Returns the index pointer for code point `code` in this index.
#[cfg(feature = "no-optimized-legacy-encoding")]
pub fn backward(code: u32) -> u8 {
    if code > 8805 || ((0x30005u32 >> (code >> 9)) & 1) == 0 {
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
