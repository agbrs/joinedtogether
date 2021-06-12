use agb::display::{tiled0::Background, HEIGHT, WIDTH};

fn num_digits_iter(mut n: u32) -> impl core::iter::Iterator<Item = u8> {
    let mut length = 0;
    core::iter::from_fn(move || {
        if n == 0 {
            length += 1;
            if length <= 1 {
                Some(0)
            } else {
                None
            }
        } else {
            length += 1;
            let c = n % 10;
            n /= 10;
            Some(c as u8)
        }
    })
}

const LEVEL_START: u16 = 12 * 28;
const NUMBERS_START: u16 = 12 * 28 + 3;
const HYPHEN: u16 = 12 * 28 + 11;
const BLANK: u16 = 11 * 28;

pub fn write_level(background: &mut Background, world: u32, level: u32) {
    let mut map = [BLANK; 20];
    let mut counter = 0;

    map[0] = LEVEL_START;
    map[1] = LEVEL_START + 1;
    map[2] = LEVEL_START + 2;

    counter += 4;

    map[counter] = world as u16 + NUMBERS_START - 1;
    counter += 1;
    map[counter] = HYPHEN;
    counter += 1;
    map[counter] = level as u16 + NUMBERS_START - 1;
    counter += 1;

    background.set_position(
        &map,
        (10_u32, 1_u32).into(),
        (-(WIDTH / 2 - counter as i32 * 8 / 2), -(HEIGHT / 2 - 4)).into(),
        BLANK,
    );
    background.draw_full_map(&map, (10_u32, 1_u32).into(), BLANK);
}
