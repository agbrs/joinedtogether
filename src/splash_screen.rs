mod splash {
    include!(concat!(env!("OUT_DIR"), "/splash.rs"));
}

pub fn show_splash_screen(agb: &mut agb::Gba) {
    let mut tiled = agb.display.video.tiled0();
    tiled.set_background_tilemap(0, &splash::TILE_DATA);
    tiled.set_background_palettes(&splash::PALETTE_DATA);
    let vblank = agb.display.vblank.get();
    let mut splash_screen_display = tiled.get_background().unwrap();

    let mut entries: [u16; 30 * 20] = [0; 30 * 20];
    for tile_id in 0..(30 * 20) {
        entries[tile_id as usize] = tile_id;
    }
    let mut input = agb::input::ButtonController::new();
    splash_screen_display.draw_full_map(&entries, (30_u32, 20_u32).into(), 0);
    splash_screen_display.show();
    loop {
        input.update();
        if input.is_just_pressed(
            agb::input::Button::A
                | agb::input::Button::B
                | agb::input::Button::START
                | agb::input::Button::SELECT,
        ) {
            break;
        }
        vblank.wait_for_VBlank();
    }
    splash_screen_display.hide();
}
