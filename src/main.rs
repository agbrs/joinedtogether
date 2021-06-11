#![no_std]
#![no_main]

mod chickenmap;

mod object_tiles {

    pub const WIZARD_TILE_START: u16 = 4;
    pub const HAT_TILE_START: u16 = 0;
    include!(concat!(env!("OUT_DIR"), "/object_sheet.rs"));
}

use agb::{
    display::{
        object::{ObjectControl, ObjectStandard, Size},
        tiled0::Background,
        HEIGHT, WIDTH,
    },
    input::{Button, ButtonController},
    number::{FixedNum, Vector2D},
};

type FixedNumberType = FixedNum<10>;

struct Entity<'a> {
    sprite: ObjectStandard<'a>,
    position: Vector2D<FixedNumberType>,
    velocity: Vector2D<FixedNumberType>,
}

impl<'a> Entity<'a> {
    fn new(object: &'a ObjectControl) -> Self {
        let sprite = object.get_object_standard();
        Entity {
            sprite,
            position: (0, 0).into(),
            velocity: (0, 0).into(),
        }
    }

    fn update_position(&mut self) {
        self.position += self.velocity;
    }
    fn commit_position(&mut self, offset: Vector2D<FixedNumberType>) {
        let position = (self.position - offset).floor();
        self.sprite.set_position(position - (8, 8).into());
        if position.x < -8 || position.x > WIDTH + 8 || position.y < -8 || position.y > HEIGHT + 8 {
            self.sprite.hide();
        } else {
            self.sprite.show();
        }
        self.sprite.commit();
    }
}

struct Map<'a> {
    layer: &'a Background,
    position: Vector2D<FixedNumberType>,
    map: &'a [u16],
    dimensions: Vector2D<u32>,
}

struct ColisionMap<'a> {
    collider: &'a [u8],
    dimensions: Vector2D<u32>,
}

impl<'a> ColisionMap<'a> {
    fn collides(&self, x: u32, y: u32) -> bool {
        let tile = self.collider[(self.dimensions.x * y + x) as usize];
        tile != 0
    }
}

#[derive(PartialEq, Eq)]
enum HatState {
    OnHead,
    Thrown,
    WizardTowards,
}

struct Player<'a> {
    wizard: Entity<'a>,
    hat: Entity<'a>,
    hat_state: HatState,
    hat_left_range: bool,
    hat_slow_counter: i32,
}

impl<'a> Player<'a> {
    fn new(controller: &'a ObjectControl) -> Self {
        let mut hat = Entity::new(controller);
        let mut wizard = Entity::new(controller);

        wizard.sprite.set_tile_id(object_tiles::WIZARD_TILE_START);
        hat.sprite.set_tile_id(object_tiles::HAT_TILE_START);
        wizard.sprite.set_sprite_size(Size::S16x16);
        hat.sprite.set_sprite_size(Size::S16x16);
        wizard.sprite.show();
        hat.sprite.show();

        wizard.sprite.commit();
        hat.sprite.commit();

        wizard.position = (WIDTH / 2, HEIGHT / 2).into();

        Player {
            wizard,
            hat,
            hat_slow_counter: 0,
            hat_state: HatState::OnHead,
            hat_left_range: false,
        }
    }

    fn update_frame(&mut self, input: &ButtonController) {
        // throw or recall
        if input.is_just_pressed(Button::A) {
            if self.hat_state == HatState::OnHead {
                let direction: Vector2D<FixedNumberType> =
                    (input.x_tri() as i32, input.y_tri() as i32).into();
                if direction != (0, 0).into() {
                    self.hat.velocity = direction.normalise() * 5;
                    self.hat_state = HatState::Thrown;
                }
            } else if self.hat_state == HatState::Thrown {
                self.hat.velocity = (0, 0).into();
                self.wizard.velocity = (0, 0).into();
                self.hat_state = HatState::WizardTowards;
            }
        }

        if self.hat_state != HatState::WizardTowards {
            let gravity: Vector2D<FixedNumberType> = (0, 1).into();
            let gravity = gravity / 16;
            self.wizard.velocity += gravity;
            self.wizard.velocity.x += FixedNumberType::new(input.x_tri() as i32) / 64;
            self.wizard.velocity = self.wizard.velocity * 62 / 64;
            self.wizard.update_position();
            self.wizard.position.y = self
                .wizard
                .position
                .y
                .clamp((-8).into(), (HEIGHT - 8).into());
        }

        match self.hat_state {
            HatState::Thrown => {
                // hat is thrown, make hat move towards wizard
                let distance_vector = self.wizard.position - self.hat.position - (0, 10).into();
                let distance = distance_vector.magnitude();
                let direction = if distance == 0.into() {
                    (0, 0).into()
                } else {
                    distance_vector / distance
                };

                if self.hat_slow_counter < 10 && self.hat.velocity.magnitude() < 2.into() {
                    self.hat.velocity = (0, 0).into();
                    self.hat_slow_counter += 1;
                } else {
                    self.hat.velocity += direction / 4;
                }
                self.hat.update_position();
                if distance > 16.into() {
                    self.hat_left_range = true;
                }
                if self.hat_left_range && distance < 16.into() {
                    self.hat_state = HatState::OnHead;
                }
            }
            HatState::OnHead => {
                // hat is on head, place hat on head
                self.hat_slow_counter = 0;
                self.hat_left_range = false;
                self.hat.position = self.wizard.position - (0, 10).into();
            }
            HatState::WizardTowards => {
                let distance_vector = self.hat.position - self.wizard.position + (0, 10).into();
                let distance = distance_vector.magnitude();
                if distance != 0.into() {
                    self.wizard.velocity += distance_vector / distance;
                }
                self.wizard.update_position();
                if distance < 8.into() {
                    self.wizard.velocity = self.wizard.velocity / 8;
                    self.hat_state = HatState::OnHead;
                }
            }
        }
    }
}

struct PlayingLevel<'a> {
    background: Map<'a>,
    input: ButtonController,
    player: Player<'a>,
}

impl<'a> PlayingLevel<'a> {
    fn open_level(
        level: &'a [u16],
        level_dimensions: Vector2D<u32>,
        object_control: &'a ObjectControl,
        background: &'a mut Background,
        input: ButtonController,
    ) -> Self {
        background.draw_full_map(level, level_dimensions);
        background.show();

        PlayingLevel {
            background: Map {
                layer: background,
                map: level,
                dimensions: level_dimensions,
                position: (0, 0).into(),
            },
            player: Player::new(object_control),
            input,
        }
    }

    fn update_frame(&mut self) {
        self.input.update();

        self.player.update_frame(&self.input);
        self.player.wizard.commit_position(self.background.position);
        self.player.hat.commit_position(self.background.position);
    }
}

#[no_mangle]
pub fn main() -> ! {
    let mut agb = agb::Gba::new();
    let mut tiled = agb.display.video.tiled0();
    let mut object = agb.display.object.get();

    tiled.set_background_palette_raw(&chickenmap::MAP_PALETTE);
    tiled.set_background_tilemap(0, &chickenmap::MAP_TILES);
    tiled.set_sprite_palettes(object_tiles::PALETTE_DATA);
    tiled.set_sprite_tilemap(object_tiles::TILE_DATA);

    let mut background = tiled.get_background().unwrap();
    object.enable();

    let mut level = PlayingLevel::open_level(
        &chickenmap::MAP_MAP,
        (32_u32, 32_u32).into(),
        &object,
        &mut background,
        agb::input::ButtonController::new(),
    );

    let vblank = agb.display.vblank.get();

    loop {
        level.update_frame();
        vblank.wait_for_VBlank();
    }
}
