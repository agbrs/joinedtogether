#![no_std]
#![no_main]

mod enemies;

pub struct Level {
    background: &'static [u16],
    foreground: &'static [u16],
    dimensions: Vector2D<u32>,
    collision: &'static [u32],
}

mod object_tiles {
    pub const WIZARD_TILE_START: u16 = 0 * 4;
    pub const WIZARD_JUMP: u16 = 4 * 4;
    pub const WIZARD_FALL_START: u16 = 5 * 4;

    pub const HAT_TILE_START: u16 = 9 * 4;
    pub const HAT_TILE_START_SECOND: u16 = 28 * 4;
    pub const HAT_TILE_START_THIRD: u16 = 38 * 4;

    pub const SLIME_IDLE_START: u16 = 19 * 4;
    pub const SLIME_JUMP_START: u16 = 20 * 4;
    pub const SLIME_SPLAT_START: u16 = 24 * 4;

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
        Priority, HEIGHT, WIDTH,
    },
    input::{self, Button, ButtonController},
    number::{FixedNum, Vector2D},
};

type FixedNumberType = FixedNum<10>;

pub struct Entity<'a> {
    sprite: ObjectStandard<'a>,
    position: Vector2D<FixedNumberType>,
    velocity: Vector2D<FixedNumberType>,
    collision_mask: Vector2D<u16>,
}

impl<'a> Entity<'a> {
    pub fn new(object: &'a ObjectControl, collision_mask: Vector2D<u16>) -> Self {
        let mut sprite = object.get_object_standard();
        sprite.set_priority(Priority::P1);
        Entity {
            sprite,
            collision_mask,
            position: (0, 0).into(),
            velocity: (0, 0).into(),
        }
    }

    fn lr_collision_at_point(&self, level: &Level, position: Vector2D<FixedNumberType>) -> bool {
        let left = (position.x - self.collision_mask.x as i32 / 2).floor() / 8;
        let right = (position.x + self.collision_mask.x as i32 / 2).floor() / 8;
        let top = (position.y - self.collision_mask.y as i32 / 2 + 1).floor() / 8;
        let bottom = (position.y + self.collision_mask.y as i32 / 2 - 1).floor() / 8;

        for x in left..=right {
            for y in top..=bottom {
                if level.collides(x, y) {
                    return true;
                }
            }
        }
        false
    }

    fn tb_collision_at_point(&self, level: &Level, position: Vector2D<FixedNumberType>) -> bool {
        let left = (position.x - self.collision_mask.x as i32 / 2 + 1).floor() / 8;
        let right = (position.x + self.collision_mask.x as i32 / 2 - 1).floor() / 8;
        let top = (position.y - self.collision_mask.y as i32 / 2).floor() / 8;
        let bottom = (position.y + self.collision_mask.y as i32 / 2).floor() / 8;

        for x in left..=right {
            for y in top..=bottom {
                if level.collides(x, y) {
                    return true;
                }
            }
        }
        false
    }

    fn killision_at_point(&self, level: &Level, position: Vector2D<FixedNumberType>) -> bool {
        let left = (position.x - self.collision_mask.x as i32 / 2).floor() / 8;
        let right = (position.x + self.collision_mask.x as i32 / 2).floor() / 8;
        let top = (position.y - self.collision_mask.y as i32 / 2).floor() / 8;
        let bottom = (position.y + self.collision_mask.y as i32 / 2).floor() / 8;

        for x in left..=right {
            for y in top..=bottom {
                if level.kills(x, y) {
                    return true;
                }
            }
        }
        false
    }

    // returns the distance actually moved
    fn update_position(&mut self, level: &Level) -> Vector2D<FixedNumberType> {
        let old_position = self.position;
        let x_velocity = (self.velocity.x, 0.into()).into();
        if !self.lr_collision_at_point(level, self.position + x_velocity) {
            self.position += x_velocity;
        }

        let y_velocity = (0.into(), self.velocity.y).into();
        if !self.tb_collision_at_point(level, self.position + y_velocity) {
            self.position += y_velocity;
        }

        self.position - old_position
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

impl<'a> Map<'a> {
    pub fn commit_position(&mut self) {
        self.background.set_position(
            self.level.foreground,
            self.level.dimensions,
            self.position.floor(),
        );
        self.foreground.set_position(
            self.level.background,
            self.level.dimensions,
            self.position.floor(),
        );
    }
}

impl Level {
    fn collides(&self, x: i32, y: i32) -> bool {
        if (x < 0 || x >= self.dimensions.x as i32) || (y < 0 || y >= self.dimensions.y as i32) {
            return true;
        }
        let pos = (self.dimensions.x as i32 * y + x) as usize;
        let tile_foreground = self.foreground[pos];
        let tile_background = self.background[pos];
        let foreground_tile_property = self.collision[tile_foreground as usize];
        foreground_tile_property == map_tiles::tilemap::COLLISION_TILE as u32
    }

    fn kills(&self, x: i32, y: i32) -> bool {
        if (x < 0 || x >= self.dimensions.x as i32) || (y < 0 || y >= self.dimensions.y as i32) {
            return true;
        }
        let pos = (self.dimensions.x as i32 * y + x) as usize;
        let tile_foreground = self.foreground[pos];
        let tile_background = self.background[pos];
        let foreground_tile_property = self.collision[tile_foreground as usize];
        foreground_tile_property == map_tiles::tilemap::KILL_TILE as u32
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum HatState {
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
    num_recalls: i8,
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
        let mut hat = Entity::new(controller, (6_u16, 6_u16).into());
        let mut wizard = Entity::new(controller, (6_u16, 14_u16).into());

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
            num_recalls: 0,
            facing: input::Tri::Zero,
        }
    }

    fn update_frame(&mut self, input: &ButtonController, timer: i32, level: &Level) {
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
                self.num_recalls += 1;
                if self.num_recalls < 3 {
                    self.hat.velocity = (0, 0).into();
                    self.wizard.velocity = (0, 0).into();
                    self.hat_state = HatState::WizardTowards;
                }
            } else if self.hat_state == HatState::WizardTowards {
                self.hat_state = HatState::Thrown;
                self.wizard.velocity /= 8;
            }
        }

        let is_on_ground = self
            .wizard
            .tb_collision_at_point(level, self.wizard.position + (0, 1).into());

        if self.hat_state != HatState::WizardTowards {
            if is_on_ground {
                self.num_recalls = 0;
            }

            if is_on_ground {
                self.wizard.velocity.x += FixedNumberType::new(input.x_tri() as i32) / 16;
                self.wizard.velocity = self.wizard.velocity * 54 / 64;
                if input.is_just_pressed(Button::B) {
                    self.wizard.velocity.y = -FixedNumberType::new(3) / 2;
                }
            } else {
                self.wizard.velocity.x += FixedNumberType::new(input.x_tri() as i32) / 64;
                self.wizard.velocity = self.wizard.velocity * 63 / 64;
                let gravity: Vector2D<FixedNumberType> = (0, 1).into();
                let gravity = gravity / 16;
                self.wizard.velocity += gravity;
            }

            self.wizard.velocity = self.wizard.update_position(level);

            if self.wizard.velocity.x.abs() > 0.into() {
                let offset = (ping_pong(timer / 16, 4)) as u16;
                self.wizard_frame = offset as u8;

                self.wizard
                    .sprite
                    .set_tile_id(object_tiles::WIZARD_TILE_START + offset * 4);
            }

            if self.wizard.velocity.y < -FixedNumberType::new(1) / 16 {
                // going up
                self.wizard_frame = 5;

                self.wizard.sprite.set_tile_id(object_tiles::WIZARD_JUMP);
            } else if self.wizard.velocity.y > FixedNumberType::new(1) / 16 {
                // going down
                let offset = ((timer / 4) % 4) as u16;
                self.wizard_frame = 0;

                self.wizard
                    .sprite
                    .set_tile_id(object_tiles::WIZARD_FALL_START + offset * 4);
            }

            if input.x_tri() != agb::input::Tri::Zero {
                self.facing = input.x_tri();
            }
        }

        let hat_base_tile = match self.num_recalls {
            0 => object_tiles::HAT_TILE_START,
            1 => object_tiles::HAT_TILE_START_SECOND,
            2 | _ => object_tiles::HAT_TILE_START_THIRD,
        };

        match self.facing {
            agb::input::Tri::Negative => {
                self.wizard.sprite.set_hflip(true);
                self.hat.sprite.set_tile_id(hat_base_tile + 4 * 5);
            }
            agb::input::Tri::Positive => {
                self.wizard.sprite.set_hflip(false);
                self.hat.sprite.set_tile_id(hat_base_tile);
            }
            _ => {}
        }

        let hat_resting_position = match self.wizard_frame {
            1 | 2 => (0, 9).into(),
            5 => (0, 10).into(),
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

                let hat_sprite_divider = match self.num_recalls {
                    0 => 1,
                    1 => 2,
                    2 | _ => 4,
                };

                let hat_sprite_offset = timer / hat_sprite_divider % 10;

                self.hat
                    .sprite
                    .set_tile_id(hat_base_tile + (hat_sprite_offset * 4) as u16);

                if self.hat_slow_counter < 10 && self.hat.velocity.magnitude() < 2.into() {
                    self.hat.velocity = (0, 0).into();
                    self.hat_slow_counter += 1;
                } else {
                    self.hat.velocity += direction / 4;
                }
                self.hat.velocity = self.hat.update_position(level);
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
                    .set_tile_id(hat_base_tile + 4 * (timer / 2 % 10) as u16);
                let distance_vector =
                    self.hat.position - self.wizard.position + hat_resting_position;
                let distance = distance_vector.magnitude();
                if distance != 0.into() {
                    let v = self.wizard.velocity.magnitude() + 1;
                    self.wizard.velocity = distance_vector / distance * v;
                }
                self.wizard.velocity = self.wizard.update_position(level);
                if distance < 16.into() {
                    self.wizard.velocity /= 8;
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

    enemies: [enemies::Enemy<'a>; 16],
}

enum UpdateState {
    Normal,
    Dead,
    Complete,
}

impl<'a> PlayingLevel<'a> {
    fn open_level(
        level: Level,
        object_control: &'a ObjectControl,
        background: &'a mut Background,
        foreground: &'a mut Background,
        input: ButtonController,
    ) -> Self {
        background.set_position(level.foreground, level.dimensions, (0, 0).into());
        background.draw_full_map(level.foreground, level.dimensions);
        background.show();

        foreground.set_position(level.background, level.dimensions, (0, 0).into());
        foreground.draw_full_map(level.background, level.dimensions);
        foreground.set_priority(Priority::P2);
        foreground.show();

        let mut e: [enemies::Enemy<'a>; 16] = Default::default();
        e[0] = enemies::Enemy::new_slime(object_control, (9 * 8, 17 * 8 + 1).into());

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
            enemies: e,
        }
    }

    fn update_frame(&mut self) -> UpdateState {
        self.timer += 1;
        self.input.update();

        self.player
            .update_frame(&self.input, self.timer, &self.background.level);

        for enemy in self.enemies.iter_mut() {
            enemy.update(
                &self.background.level,
                self.player.wizard.position,
                self.player.hat_state,
                self.timer,
            );
        }

        self.background.position = self.get_next_map_position();
        self.background.commit_position();

        self.player.wizard.commit_position(self.background.position);
        self.player.hat.commit_position(self.background.position);

        for enemy in self.enemies.iter_mut() {
            enemy.commit(self.background.position);
        }

        if self
            .player
            .wizard
            .killision_at_point(&self.background.level, self.player.wizard.position)
        {
            UpdateState::Dead
        } else {
            UpdateState::Normal
        }
    }

    fn get_next_map_position(&self) -> Vector2D<FixedNumberType> {
        // want to ensure the player and the hat are visible if possible, so try to position the map
        // so the centre is at the average position. But give the player some extra priority
        let hat_pos = self.player.hat.position;
        let player_pos = self.player.wizard.position;

        let new_target_position = (hat_pos + player_pos * 3) / 4;

        let screen: Vector2D<FixedNumberType> = (WIDTH, HEIGHT).into();
        let half_screen = screen / 2;
        let current_centre = self.background.position + half_screen;

        let mut target_position = ((current_centre * 3 + new_target_position) / 4) - half_screen;

        target_position.x = target_position.x.clamp(
            0.into(),
            ((self.background.level.dimensions.x * 8 - (WIDTH as u32)) as i32).into(),
        );
        target_position.y = target_position.y.clamp(
            0.into(),
            ((self.background.level.dimensions.y * 8 - (HEIGHT as u32)) as i32).into(),
        );

        target_position
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

    let vblank = agb.display.vblank.get();

    loop {
        let mut level = PlayingLevel::open_level(
            Level {
                background: &map_tiles::level1::TILEMAP,
                foreground: &map_tiles::level1::BACKGROUND,
                dimensions: (map_tiles::level1::WIDTH, map_tiles::level1::HEIGHT).into(),
                collision: &map_tiles::tilemap::TILE_DATA,
            },
            &object,
            &mut background,
            &mut foreground,
            agb::input::ButtonController::new(),
        );
        loop {
            match level.update_frame() {
                UpdateState::Normal => {}
                UpdateState::Dead => {
                    break;
                }
                UpdateState::Complete => {}
            }
            vblank.wait_for_VBlank();
        }
    }
}
