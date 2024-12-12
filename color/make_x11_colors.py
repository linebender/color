# Copyright 2024 the Color Authors
# SPDX-License-Identifier: Apache-2.0 OR MIT

# A utility to create a minimal perfect hash lookup table for
# the x11 palette colors.
#
# This utility has been adapted from <https://github.com/unicode-rs/unicode-normalization/pull/37>.
#
# See Steve Hanov's blog
# [Throw away the keys: Easy, Minimal Perfect Hashing](https://stevehanov.ca/blog/?id=119)
# for the basic technique.

colors = [
    ("aliceblue", (240, 248, 255, 255)),
    ("antiquewhite", (250, 235, 215, 255)),
    ("aqua", (0, 255, 255, 255)),
    ("aquamarine", (127, 255, 212, 255)),
    ("azure", (240, 255, 255, 255)),
    ("beige", (245, 245, 220, 255)),
    ("bisque", (255, 228, 196, 255)),
    ("black", (0, 0, 0, 255)),
    ("blanchedalmond", (255, 235, 205, 255)),
    ("blue", (0, 0, 255, 255)),
    ("blueviolet", (138, 43, 226, 255)),
    ("brown", (165, 42, 42, 255)),
    ("burlywood", (222, 184, 135, 255)),
    ("cadetblue", (95, 158, 160, 255)),
    ("chartreuse", (127, 255, 0, 255)),
    ("chocolate", (210, 105, 30, 255)),
    ("coral", (255, 127, 80, 255)),
    ("cornflowerblue", (100, 149, 237, 255)),
    ("cornsilk", (255, 248, 220, 255)),
    ("crimson", (220, 20, 60, 255)),
    ("cyan", (0, 255, 255, 255)),
    ("darkblue", (0, 0, 139, 255)),
    ("darkcyan", (0, 139, 139, 255)),
    ("darkgoldenrod", (184, 134, 11, 255)),
    ("darkgray", (169, 169, 169, 255)),
    ("darkgreen", (0, 100, 0, 255)),
    ("darkkhaki", (189, 183, 107, 255)),
    ("darkmagenta", (139, 0, 139, 255)),
    ("darkolivegreen", (85, 107, 47, 255)),
    ("darkorange", (255, 140, 0, 255)),
    ("darkorchid", (153, 50, 204, 255)),
    ("darkred", (139, 0, 0, 255)),
    ("darksalmon", (233, 150, 122, 255)),
    ("darkseagreen", (143, 188, 143, 255)),
    ("darkslateblue", (72, 61, 139, 255)),
    ("darkslategray", (47, 79, 79, 255)),
    ("darkturquoise", (0, 206, 209, 255)),
    ("darkviolet", (148, 0, 211, 255)),
    ("deeppink", (255, 20, 147, 255)),
    ("deepskyblue", (0, 191, 255, 255)),
    ("dimgray", (105, 105, 105, 255)),
    ("dodgerblue", (30, 144, 255, 255)),
    ("firebrick", (178, 34, 34, 255)),
    ("floralwhite", (255, 250, 240, 255)),
    ("forestgreen", (34, 139, 34, 255)),
    ("fuchsia", (255, 0, 255, 255)),
    ("gainsboro", (220, 220, 220, 255)),
    ("ghostwhite", (248, 248, 255, 255)),
    ("gold", (255, 215, 0, 255)),
    ("goldenrod", (218, 165, 32, 255)),
    ("gray", (128, 128, 128, 255)),
    ("green", (0, 128, 0, 255)),
    ("greenyellow", (173, 255, 47, 255)),
    ("honeydew", (240, 255, 240, 255)),
    ("hotpink", (255, 105, 180, 255)),
    ("indianred", (205, 92, 92, 255)),
    ("indigo", (75, 0, 130, 255)),
    ("ivory", (255, 255, 240, 255)),
    ("khaki", (240, 230, 140, 255)),
    ("lavender", (230, 230, 250, 255)),
    ("lavenderblush", (255, 240, 245, 255)),
    ("lawngreen", (124, 252, 0, 255)),
    ("lemonchiffon", (255, 250, 205, 255)),
    ("lightblue", (173, 216, 230, 255)),
    ("lightcoral", (240, 128, 128, 255)),
    ("lightcyan", (224, 255, 255, 255)),
    ("lightgoldenrodyellow", (250, 250, 210, 255)),
    ("lightgray", (211, 211, 211, 255)),
    ("lightgreen", (144, 238, 144, 255)),
    ("lightpink", (255, 182, 193, 255)),
    ("lightsalmon", (255, 160, 122, 255)),
    ("lightseagreen", (32, 178, 170, 255)),
    ("lightskyblue", (135, 206, 250, 255)),
    ("lightslategray", (119, 136, 153, 255)),
    ("lightsteelblue", (176, 196, 222, 255)),
    ("lightyellow", (255, 255, 224, 255)),
    ("lime", (0, 255, 0, 255)),
    ("limegreen", (50, 205, 50, 255)),
    ("linen", (250, 240, 230, 255)),
    ("magenta", (255, 0, 255, 255)),
    ("maroon", (128, 0, 0, 255)),
    ("mediumaquamarine", (102, 205, 170, 255)),
    ("mediumblue", (0, 0, 205, 255)),
    ("mediumorchid", (186, 85, 211, 255)),
    ("mediumpurple", (147, 112, 219, 255)),
    ("mediumseagreen", (60, 179, 113, 255)),
    ("mediumslateblue", (123, 104, 238, 255)),
    ("mediumspringgreen", (0, 250, 154, 255)),
    ("mediumturquoise", (72, 209, 204, 255)),
    ("mediumvioletred", (199, 21, 133, 255)),
    ("midnightblue", (25, 25, 112, 255)),
    ("mintcream", (245, 255, 250, 255)),
    ("mistyrose", (255, 228, 225, 255)),
    ("moccasin", (255, 228, 181, 255)),
    ("navajowhite", (255, 222, 173, 255)),
    ("navy", (0, 0, 128, 255)),
    ("oldlace", (253, 245, 230, 255)),
    ("olive", (128, 128, 0, 255)),
    ("olivedrab", (107, 142, 35, 255)),
    ("orange", (255, 165, 0, 255)),
    ("orangered", (255, 69, 0, 255)),
    ("orchid", (218, 112, 214, 255)),
    ("palegoldenrod", (238, 232, 170, 255)),
    ("palegreen", (152, 251, 152, 255)),
    ("paleturquoise", (175, 238, 238, 255)),
    ("palevioletred", (219, 112, 147, 255)),
    ("papayawhip", (255, 239, 213, 255)),
    ("peachpuff", (255, 218, 185, 255)),
    ("peru", (205, 133, 63, 255)),
    ("pink", (255, 192, 203, 255)),
    ("plum", (221, 160, 221, 255)),
    ("powderblue", (176, 224, 230, 255)),
    ("purple", (128, 0, 128, 255)),
    ("rebeccapurple", (102, 51, 153, 255)),
    ("red", (255, 0, 0, 255)),
    ("rosybrown", (188, 143, 143, 255)),
    ("royalblue", (65, 105, 225, 255)),
    ("saddlebrown", (139, 69, 19, 255)),
    ("salmon", (250, 128, 114, 255)),
    ("sandybrown", (244, 164, 96, 255)),
    ("seagreen", (46, 139, 87, 255)),
    ("seashell", (255, 245, 238, 255)),
    ("sienna", (160, 82, 45, 255)),
    ("silver", (192, 192, 192, 255)),
    ("skyblue", (135, 206, 235, 255)),
    ("slateblue", (106, 90, 205, 255)),
    ("slategray", (112, 128, 144, 255)),
    ("snow", (255, 250, 250, 255)),
    ("springgreen", (0, 255, 127, 255)),
    ("steelblue", (70, 130, 180, 255)),
    ("tan", (210, 180, 140, 255)),
    ("teal", (0, 128, 128, 255)),
    ("thistle", (216, 191, 216, 255)),
    ("tomato", (255, 99, 71, 255)),
    ("transparent", (0, 0, 0, 0)),
    ("turquoise", (64, 224, 208, 255)),
    ("violet", (238, 130, 238, 255)),
    ("wheat", (245, 222, 179, 255)),
    ("white", (255, 255, 255, 255)),
    ("whitesmoke", (245, 245, 245, 255)),
    ("yellow", (255, 255, 0, 255)),
    ("yellowgreen", (154, 205, 50, 255)),
]

def weak_hash_string(s):
    mask_32 = 0xffffffff
    h = 0
    for char in s:
        h = (9 * h + ord(char)) & mask_32
    return h

# Guaranteed to be less than n.
def weak_hash(s, salt, n):
    x = weak_hash_string(s[0])
    # This is hash based on the theory that multiplication is efficient
    mask_32 = 0xffffffff
    y = ((x + salt) * 2654435769) & mask_32
    y ^= x
    return (y * n) >> 32

# Compute minimal perfect hash function, d can be either a dict or list of keys.
def minimal_perfect_hash(d):
    n = len(d)
    buckets = dict((h, []) for h in range(n))
    for key in d:
        h = weak_hash(key, 0, n)
        buckets[h].append(key)
    bsorted = [(len(buckets[h]), h) for h in range(n)]
    bsorted.sort(reverse = True)
    claimed = [False] * n
    salts = [0] * n
    keys = [0] * n
    for (bucket_size, h) in bsorted:
        # Note: the traditional perfect hashing approach would also special-case
        # bucket_size == 1 here and assign any empty slot, rather than iterating
        # until rehash finds an empty slot. But we're not doing that so we can
        # avoid the branch.
        if bucket_size == 0:
            break
        else:
            for salt in range(1, 32768):
                rehashes = [weak_hash(key, salt, n) for key in buckets[h]]
                # Make sure there are no rehash collisions within this bucket.
                if all(not claimed[hash] for hash in rehashes):
                    if len(set(rehashes)) < bucket_size:
                        continue
                    salts[h] = salt
                    for key in buckets[h]:
                        rehash = weak_hash(key, salt, n)
                        claimed[rehash] = True
                        keys[rehash] = key
                    break
            if salts[h] == 0:
                print("minimal perfect hashing failed")
                # Note: if this happens (because of unfortunate data), then there are
                # a few things that could be done. First, the hash function could be
                # tweaked. Second, the bucket order could be scrambled (especially the
                # singletons). Right now, the buckets are sorted, which has the advantage
                # of being deterministic.
                #
                # As a more extreme approach, the singleton bucket optimization could be
                # applied (give the direct address for singleton buckets, rather than
                # relying on a rehash). That is definitely the more standard approach in
                # the minimal perfect hashing literature, but in testing the branch was a
                # significant slowdown.
                exit(1)
    return (salts, keys)

(salts, keys) = minimal_perfect_hash(colors)
n = len(colors)
print("""// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

// This file was auto-generated by make_x11_colors.py. Do not hand-edit.
""")
print(f"const SALTS: [u8; {n}] = [")
obuf = "   "
for salt in salts:
    word = f" {salt},"
    if len(obuf) + len(word) >= 100:
        print(obuf)
        obuf = "   "
    obuf += word
if len(obuf) > 3:
    print(obuf)
print("];")
print()

print(f"pub(crate) const NAMES: [&str; {n}] = [")
for (name, rgba) in keys:
    print(f'    "{name}",')
print("];")
print(f"""
/// RGBA8 color components of the named X11 colors, in the same order as [`NAMES`].
///
/// Use [`lookup_palette_index`] to efficiently find the color components for a given color name
/// string.
pub(crate) const COLORS: [[u8; 4]; {n}] = [""")
for (name, rgba) in keys:
    print(f'    {list(rgba)},')
print("];")
print("""
/// Hash the 32 bit key into a value less than `n`, adding salt.
///
/// This is basically the weakest hash we can get away with that
/// still distinguishes all the values.
#[inline]
fn weak_hash(key: u32, salt: u32, n: usize) -> usize {
    let y = key.wrapping_add(salt).wrapping_mul(2654435769);
    let y = y ^ key;
    (((y as u64) * (n as u64)) >> 32) as usize
}

/// Given a named color (e.g., "red", "mediumorchid"), returns the index of that color into
/// [`COLORS`] and [`NAMES`].
pub(crate) fn lookup_palette_index(s: &str) -> Option<usize> {
    let mut key = 0_u32;
    for b in s.as_bytes() {
        key = key.wrapping_mul(9).wrapping_add(*b as u32);
    }
    let salt = SALTS[weak_hash(key, 0, SALTS.len())] as u32;
    let ix = weak_hash(key, salt, SALTS.len());
    if s == NAMES[ix] {
        Some(ix)
    } else {
        None
    }
}""")
