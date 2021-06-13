use agb::sound::mixer::{Mixer, SoundChannel};

mod music_data {
    // From the open game art page:
    //
    // USING THE LOOPED VERSION:
    // 1. Play the intro.
    // 2. When the intro reaches approximately 11.080 seconds, trigger the main loop and let the intro finish underneath it.
    // 3. Re-trigger the main loop every time it reaches 1 minute 26.080 seconds, and let the old instance finish underneath the new one.
    pub const INTRO_MUSIC: &[u8] =
        include_bytes!("../sfx/Otto Halmén - Sylvan Waltz (loop intro).raw");
    pub const LOOP: &[u8] = include_bytes!("../sfx/Otto Halmén - Sylvan Waltz (loop main).raw");

    // These are based on the instructions above and a frame rate of 59.73Hz
    pub const TRIGGER_MUSIC_POINT: i32 = 662;
    pub const LOOP_MUSIC: i32 = 5141;
}

mod effects {
    const WOOSH1: &[u8] = include_bytes!("../sfx/woosh1.raw");
    const WOOSH2: &[u8] = include_bytes!("../sfx/woosh2.raw");
    const WOOSH3: &[u8] = include_bytes!("../sfx/woosh3.raw");

    pub const WHOOSHES: &[&[u8]] = &[WOOSH1, WOOSH2, WOOSH3];

    pub const CATCH: &[u8] = include_bytes!("../sfx/catch.raw");
}

pub struct MusicBox {
    frame: i32,
}

impl MusicBox {
    pub fn new() -> Self {
        MusicBox { frame: 0 }
    }

    pub fn after_blank(&mut self, mixer: &mut Mixer) {
        if self.frame == 0 {
            // play the introduction
            mixer.play_sound(SoundChannel::new(music_data::INTRO_MUSIC));
        } else if self.frame == music_data::TRIGGER_MUSIC_POINT
            || (self.frame - music_data::TRIGGER_MUSIC_POINT) % music_data::LOOP_MUSIC == 0
        {
            mixer.play_sound(SoundChannel::new(music_data::LOOP));
        }

        self.frame += 1;
    }
}

pub struct SfxPlayer<'a> {
    mixer: &'a mut Mixer,
    frame: i32,
}

impl<'a> SfxPlayer<'a> {
    pub fn new(mixer: &'a mut Mixer, music_box: &MusicBox) -> Self {
        SfxPlayer {
            mixer,
            frame: music_box.frame,
        }
    }

    pub fn catch(&mut self) {
        self.mixer.play_sound(SoundChannel::new(effects::CATCH));
    }

    pub fn throw(&mut self) {
        self.play_random(effects::WHOOSHES);
    }

    fn play_random(&mut self, effect: &[&'static [u8]]) {
        self.mixer.play_sound(SoundChannel::new(
            effect[(self.frame as usize) % effect.len()],
        ));
    }
}
