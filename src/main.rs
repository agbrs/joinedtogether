#![no_std]
#![no_main]

struct Level {
    background: &'static [u16],
    foreground: &'static [u16],
    dimensions: Vector2D<u32>,
    collision: &'static [u8],
}

mod object_tiles {
    pub const WIZARD_TILE_START: u16 = 0 * 4;
    pub const WIZARD_JUMP: u16 = 4 * 4;
    pub const WIZARD_FALL_START: u16 = 5 * 4;
    pub const HAT_TILE_START: u16 = 9 * 4;
    include!(concat!(env!("OUT_DIR"), "/object_sheet.rs"));
}

mod map_tiles {
    pub mod level1 {
        include!(concat!(env!("OUT_DIR"), "/level1.json.rs"));
    }

    pub mod tilemap {
        include!(concat!(env!("OUT_DIR"), "/tilemap.rs"));
    }

    pub mod tiles {
        include!(concat!(env!("OUT_DIR"), "/tile_sheet.rs"));
    }
}

use agb::{
    display::{
        object::{ObjectControl, ObjectStandard, Size},
        tiled0::Background,
        HEIGHT, WIDTH,
    },
    input::{self, Button, ButtonController},
    number::{FixedNum, Vector2D},
};
use object_tiles::HAT_TILE_START;

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
    background: &'a mut Background,
    foreground: &'a mut Background,
    position: Vector2D<FixedNumberType>,
    level: Level,
}

impl Level {
    fn collides(&self, x: u32, y: u32) -> bool {
        let tile = self.collision[(self.dimensions.x * y + x) as usize];
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
    wizard_frame: u8,
    facing: input::Tri,
}

fn ping_pong(i: i32, n: i32) -> i32 {
    let cycle = 2 * (n - 1);
    let i = i % cycle;
    if i >= n {
        cycle - i
    } else {
        i
    }
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
            wizard_frame: 0,
            facing: input::Tri::Zero,
        }
    }

    fn update_frame(&mut self, input: &ButtonController, timer: i32) {
        // throw or recall
        if input.is_just_pressed(Button::A) {
            if self.hat_state == HatState::OnHead {
                let direction: Vector2D<FixedNumberType> =
                    (input.x_tri() as i32, input.y_tri() as i32).into();
                if direction != (0, 0).into() {
                    let mut velocity = direction.normalise() * 5;
                    if velocity.y > 0.into() {
                        velocity.y *= FixedNumberType::new(4) / 3;
                    }
                    self.hat.velocity = velocity;
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

            if self.wizard.velocity.x.abs() > FixedNumberType::new(1) / 16 {
                let offset = (ping_pong(timer / 16, 4)) as u16;
                self.wizard_frame = offset as u8;

                self.wizard
                    .sprite
                    .set_tile_id(object_tiles::WIZARD_TILE_START + offset * 4);
            }

            if self.wizard.velocity.y < FixedNumberType::new(1) / 16 {
                // going up
                self.wizard_frame = 0;

                self.wizard
                    .sprite
                    .set_tile_id(object_tiles::WIZARD_FALL_START);
            } /*else if self.wizard.velocity.y > FixedNumberType::new(1) / 16 {
                  // going down
                  let offset = ((timer / 8) % 4) as u16;
                  self.wizard_frame = 0;

                  self.wizard
                      .sprite
                      .set_tile_id(object_tiles::WIZARD_FALL_START + offset * 4);
              }*/

            if input.x_tri() != agb::input::Tri::Zero {
                self.facing = input.x_tri();
            }

            self.wizard.velocity = self.wizard.velocity * 62 / 64;
            self.wizard.update_position();
            self.wizard.position.y = self
                .wizard
                .position
                .y
                .clamp((-8).into(), (HEIGHT - 8).into());
        }

        match self.facing {
            agb::input::Tri::Negative => {
                self.wizard.sprite.set_hflip(true);
                self.hat
                    .sprite
                    .set_tile_id(object_tiles::HAT_TILE_START + 4 * 5);
            }
            agb::input::Tri::Positive => {
                self.wizard.sprite.set_hflip(false);
                self.hat.sprite.set_tile_id(object_tiles::HAT_TILE_START);
            }
            _ => {}
        }

        let hat_resting_position = match self.wizard_frame {
            1 | 2 => (0, 9).into(),
            _ => (0, 8).into(),
        };

        match self.hat_state {
            HatState::Thrown => {
                // hat is thrown, make hat move towards wizard
                let distance_vector =
                    self.wizard.position - self.hat.position - hat_resting_position;
                let distance = distance_vector.magnitude();
                let direction = if distance == 0.into() {
                    (0, 0).into()
                } else {
                    distance_vector / distance
                };

                self.hat
                    .sprite
                    .set_tile_id(object_tiles::HAT_TILE_START + 4 * (timer / 2 % 10) as u16);

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
                self.hat.position = self.wizard.position - hat_resting_position;
            }
            HatState::WizardTowards => {
                self.hat
                    .sprite
                    .set_tile_id(object_tiles::HAT_TILE_START + 4 * (timer / 2 % 10) as u16);
                let distance_vector =
                    self.hat.position - self.wizard.position + hat_resting_position;
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
    timer: i32,
    background: Map<'a>,
    input: ButtonController,
    player: Player<'a>,
}

impl<'a> PlayingLevel<'a> {
    fn open_level(
        level: Level,
        object_control: &'a ObjectControl,
        background: &'a mut Background,
        foreground: &'a mut Background,
        input: ButtonController,
    ) -> Self {
        background.draw_full_map(level.background, level.dimensions);
        background.show();

        PlayingLevel {
            timer: 0,
            background: Map {
                background,
                foreground,
                level,
                position: (0, 0).into(),
            },
            player: Player::new(object_control),
            input,
        }
    }

    fn update_frame(&mut self) {
        self.timer += 1;
        self.input.update();

        self.player.update_frame(&self.input, self.timer);

        self.player.wizard.commit_position(self.background.position);
        self.player.hat.commit_position(self.background.position);
    }
}

#[no_mangle]
pub fn main() -> ! {
    let mut agb = agb::Gba::new();
    let mut tiled = agb.display.video.tiled0();
    let mut object = agb.display.object.get();

    tiled.set_background_palettes(&map_tiles::tiles::PALETTE_DATA);
    tiled.set_background_tilemap(0, &map_tiles::tiles::TILE_DATA);
    tiled.set_sprite_palettes(object_tiles::PALETTE_DATA);
    tiled.set_sprite_tilemap(object_tiles::TILE_DATA);

    let mut background = tiled.get_background().unwrap();
    let mut foreground = tiled.get_background().unwrap();
    object.enable();

    let mut level = PlayingLevel::open_level(
        Level {
            foreground: &map_tiles::level1::TILEMAP,
            background: &map_tiles::level1::BACKGROUND,
            dimensions: (map_tiles::level1::WIDTH, map_tiles::level1::HEIGHT).into(),
            collision: &[],
        },
        &object,
        &mut background,
        &mut foreground,
        agb::input::ButtonController::new(),
    );

    let vblank = agb.display.vblank.get();

    loop {
        level.update_frame();
        vblank.wait_for_VBlank();
    }
}
