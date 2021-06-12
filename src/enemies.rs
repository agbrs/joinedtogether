use super::{object_tiles, Entity, FixedNumberType, Level};
use agb::{
    display::object::{ObjectControl, Size},
    number::Vector2D,
};

enum UpdateState {
    Nothing,
    Remove,
}

pub enum Enemy<'a> {
    Slime(Slime<'a>),
    Empty,
}

impl<'a> Default for Enemy<'a> {
    fn default() -> Self {
        Enemy::Empty
    }
}

struct EnemyInfo<'a> {
    entity: Entity<'a>,
}

impl<'a> EnemyInfo<'a> {
    fn new(
        object: &'a ObjectControl,
        start_pos: Vector2D<FixedNumberType>,
        collision: Vector2D<u16>,
    ) -> Self {
        let mut enemy_info = EnemyInfo {
            entity: Entity::new(object, collision),
        };
        enemy_info.entity.position = start_pos;
        enemy_info
    }
}

enum SlimeState {
    Idle,
    Jumping(i32), // the start frame of the jumping animation
    Dying(i32),   // the start frame of the dying animation
}

pub struct Slime<'a> {
    enemy_info: EnemyInfo<'a>,
    state: SlimeState,
}

impl<'a> Slime<'a> {
    pub fn new(object: &'a ObjectControl, start_pos: Vector2D<FixedNumberType>) -> Self {
        let mut slime = Slime {
            enemy_info: EnemyInfo::new(object, start_pos, (16u16, 16u16).into()),
            state: SlimeState::Idle,
        };

        slime.enemy_info.entity.sprite.set_sprite_size(Size::S16x16);

        slime
    }

    pub fn update(
        &mut self,
        level: &Level,
        player_pos: Vector2D<FixedNumberType>,
        timer: i32,
    ) -> UpdateState {
        match self.state {
            SlimeState::Idle => {
                let offset = (timer / 64 % 2) * 4;
                self.enemy_info
                    .entity
                    .sprite
                    .set_tile_id(object_tiles::SLIME_IDLE_START + offset as u16);

                if (self.enemy_info.entity.position - player_pos).magnitude_squared() < 128.into() {
                    self.state = SlimeState::Jumping(timer);
                }
            }
            SlimeState::Jumping(jumping_start_frame) => {
                let offset = ((timer - jumping_start_frame) / 16) * 4;

                if offset >= 7 {
                    self.enemy_info.entity.velocity = (0, 0).into();
                    self.state = SlimeState::Idle;
                } else {
                    let sprite_offset = if offset >= 4 { 7 - offset } else { offset };

                    self.enemy_info
                        .entity
                        .sprite
                        .set_tile_id(object_tiles::SLIME_JUMP_START + offset as u16);
                }
            }
            SlimeState::Dying(dying_start_frame) => {
                let offset = ((timer - dying_start_frame) / 16) * 4;

                if offset >= 4 {
                    return UpdateState::Remove;
                }

                self.enemy_info
                    .entity
                    .sprite
                    .set_tile_id(object_tiles::SLIME_SPLAT_START + offset as u16);
            }
        }

        UpdateState::Nothing
    }
}
