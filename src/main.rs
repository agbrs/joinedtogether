#![no_std]
#![no_main]

use agb::{
    display::object::ObjectStandard,
    number::{FixedNum, Vector2D},
};

type FixedNumberType = FixedNum<10>;

struct Entity<'a> {
    sprite: ObjectStandard<'a>,
    position: Vector2D<FixedNumberType>,
    velocity: Vector2D<FixedNumberType>,
}

#[no_mangle]
pub fn main() -> ! {
    let mut agb = agb::Gba::new();
    let mut tiled = agb.display.video.tiled0();
    let mut object = agb.display.object.get();

    object.enable();

    let vblank = agb.display.vblank.get();

    loop {
        vblank.wait_for_VBlank();
    }
}
