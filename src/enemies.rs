use super::{object_tiles, Entity, FixedNumberType, HatState, Level};
use agb::{
    display::object::{ObjectControl, Size},
    number::{Rect, Vector2D},
};

enum UpdateState {
    Nothing,
    KillPlayer,
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

pub enum EnemyUpdateState {
    None,
    KillPlayer,
}

impl<'a> Enemy<'a> {
    pub fn new_slime(object: &'a ObjectControl, start_pos: Vector2D<FixedNumberType>) -> Self {
        Enemy::Slime(Slime::new(object, start_pos))
    }

    pub fn update(
        &mut self,
        level: &Level,
        player_pos: Vector2D<FixedNumberType>,
        hat_state: HatState,
        timer: i32,
    ) -> EnemyUpdateState {
        let update_state = match self {
            Enemy::Slime(slime) => slime.update(level, player_pos, hat_state, timer),
            Enemy::Empty => UpdateState::Nothing,
        };

        match update_state {
            UpdateState::Remove => {
                *self = Enemy::Empty;
                EnemyUpdateState::None
            }
            UpdateState::KillPlayer => EnemyUpdateState::KillPlayer,
            UpdateState::Nothing => EnemyUpdateState::None,
        }
    }

    pub fn commit(&mut self, background_offset: Vector2D<FixedNumberType>) {
        match self {
            Enemy::Slime(slime) => slime.commit(background_offset),
            Enemy::Empty => {}
        }
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

    fn update(&mut self, level: &Level) {
        self.entity.update_position(level);
    }

    fn commit(&mut self, background_offset: Vector2D<FixedNumberType>) {
        self.entity.commit_position(background_offset);
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
    fn new(object: &'a ObjectControl, start_pos: Vector2D<FixedNumberType>) -> Self {
        let mut slime = Slime {
            enemy_info: EnemyInfo::new(object, start_pos, (14u16, 14u16).into()),
            state: SlimeState::Idle,
        };

        slime.enemy_info.entity.sprite.set_sprite_size(Size::S16x16);

        slime
    }

    fn update(
        &mut self,
        level: &Level,
        player_pos: Vector2D<FixedNumberType>,
        hat_state: HatState,
        timer: i32,
    ) -> UpdateState {
        let player_has_collided =
            (self.enemy_info.entity.position - player_pos).magnitude_squared() < (16 * 16).into();

        match self.state {
            SlimeState::Idle => {
                let offset = (timer / 16 % 2) * 4;
                self.enemy_info
                    .entity
                    .sprite
                    .set_tile_id(object_tiles::SLIME_IDLE_START + offset as u16);

                if (self.enemy_info.entity.position - player_pos).magnitude_squared()
                    < (64 * 64).into()
                {
                    self.state = SlimeState::Jumping(timer);

                    let x_vel: FixedNumberType =
                        if self.enemy_info.entity.position.x > player_pos.x {
                            -1
                        } else {
                            1
                        }
                        .into();

                    self.enemy_info.entity.velocity = (x_vel / 4, 0.into()).into();
                }

                if player_has_collided {
                    if hat_state == HatState::WizardTowards {
                        self.state = SlimeState::Dying(timer);
                    } else {
                        return UpdateState::KillPlayer;
                    }
                }
            }
            SlimeState::Jumping(jumping_start_frame) => {
                let offset = (timer - jumping_start_frame) / 4;

                if offset >= 7 {
                    self.enemy_info.entity.velocity = (0, 0).into();
                    self.state = SlimeState::Idle;
                } else {
                    let sprite_offset = if offset >= 4 { 7 - offset } else { offset };

                    self.enemy_info
                        .entity
                        .sprite
                        .set_tile_id(object_tiles::SLIME_JUMP_START + (sprite_offset * 4) as u16);
                }

                if player_has_collided {
                    if hat_state == HatState::WizardTowards {
                        self.state = SlimeState::Dying(timer);
                    } else {
                        return UpdateState::KillPlayer;
                    }
                }
            }
            SlimeState::Dying(dying_start_frame) => {
                let offset = (timer - dying_start_frame) / 4;
                self.enemy_info.entity.velocity = (0, 0).into();

                if offset >= 4 {
                    return UpdateState::Remove;
                }

                self.enemy_info
                    .entity
                    .sprite
                    .set_tile_id(object_tiles::SLIME_SPLAT_START + (offset * 4) as u16);
            }
        }

        self.enemy_info.update(level);

        UpdateState::Nothing
    }

    fn commit(&mut self, background_offset: Vector2D<FixedNumberType>) {
        self.enemy_info.commit(background_offset);
    }
}
