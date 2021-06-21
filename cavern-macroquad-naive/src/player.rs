use macroquad::prelude::{collections::storage, is_key_down, is_key_pressed, KeyCode, Texture2D};

use crate::{
    actor::{Actor, Anchor},
    collide_actor::CollideActor,
    gravity_actor::{GravityActor, GRAVITY_ACTOR_DEFAULT_ANCHOR},
    orb::Orb,
    resources::Resources,
    HEIGHT, WIDTH,
};

pub struct Player {
    pub lives: i32,
    pub score: i32,
    pub direction_x: i32, // -1 = left, 1 = right
    pub fire_timer: i32,
    pub hurt_timer: i32,
    pub health: i32,
    pub blowing_orb: Option<Orb>,

    // Actor trait
    pub x: i32,
    pub y: i32,
    pub image: Texture2D,
    pub anchor: Anchor,

    // GravityActor trait
    pub vel_y: i32,
    pub landed: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            lives: 2,
            score: 0,
            direction_x: 0,
            fire_timer: 0,
            hurt_timer: 0,
            health: 0,
            blowing_orb: None,

            x: 0,
            y: 0,
            image: storage::get::<Resources>().blank_texture,
            anchor: GRAVITY_ACTOR_DEFAULT_ANCHOR,

            vel_y: 0,
            landed: false,
        }
    }

    pub fn reset(&mut self) {
        self.x = WIDTH / 2;
        self.y = 100;
        self.vel_y = 0;
        self.direction_x = 1; // -1 = left, 1 = right
        self.fire_timer = 0;
        self.hurt_timer = 100; // Invulnerable for this many frames
        self.health = 3;
        self.blowing_orb = None;
    }

    pub fn update(&mut self, orbs: &mut Vec<Orb>, grid: &[&str], game_timer: i32) {
        // Call GravityActor.update - parameter is whether we want to perform collision detection as we fall. If health
        // is zero, we want the player to just fall out of the level
        GravityActor::update(self, self.health > 0, grid);

        self.fire_timer -= 1;
        self.hurt_timer -= 1;

        // Get keyboard input. dx represents the direction the player is facing
        // Rust: In the original code, this is (inappropriately but functionally) inside the else block, which, in static
        // languages, is out of scope.
        let mut dx = 0;

        if self.landed {
            // Hurt timer starts at 200, but drops to 100 once the player has landed
            self.hurt_timer = self.hurt_timer.min(100);
        }

        if self.hurt_timer > 100 {
            // We've just been hurt. Either carry out the sideways motion from being knocked by a bolt, or if health is
            // zero, we're dropping out of the level, so check for our sprite reaching a certain Y coordinate before
            // reducing our lives count and responding the player. We check for the Y coordinate being the screen height
            // plus 50%, rather than simply the screen height, because the former effectively gives us a short delay
            // before the player respawns.
            if self.health > 0 {
                self.move_(self.direction_x, 0, 4, grid);
            } else {
                if self.top() >= (HEIGHT as f32 * 1.5) as i32 {
                    self.lives -= 1;
                    self.reset();
                }
            }
        } else {
            // We're not hurt
            if is_key_down(KeyCode::Left) {
                dx = -1;
            } else if is_key_down(KeyCode::Right) {
                dx = 1;
            }

            if dx != 0 {
                self.direction_x = dx;

                // If we haven't just fired an orb, carry out horizontal movement
                if self.fire_timer < 10 {
                    self.move_(dx, 0, 4, grid);
                }
            }

            // Do we need to create a new orb? Space must have been pressed and released, the minimum time between
            // orbs must have passed, and there is a limit of 5 orbs.
            if is_key_pressed(KeyCode::Space) && self.fire_timer <= 0 && orbs.len() < 5 {
                // x position will be 38 pixels in front of the player position, while ensuring it is within the
                // bounds of the level
                let x = (self.x() + self.direction_x * 38).clamp(70, 730);
                let y = self.y() - 35;
                self.blowing_orb = Some(Orb::new(x, y, self.direction_x));
                orbs.push(self.blowing_orb.unwrap().clone());
                eprint!("WRITEME: play_sound inside Player#update()");
                // game.play_sound("blow", 4);
                self.fire_timer = 20;
            }

            if is_key_down(KeyCode::Up) && self.vel_y == 0 && self.landed {
                // Jump
                self.vel_y = -16;
                self.landed = false;
                eprint!("WRITEME: play_sound inside Player#update()");
                // game.play_sound("jump");
            }
        }

        // Holding down space causes the current orb (if there is one) to be blown further
        if is_key_down(KeyCode::Space) {
            if let Some(blowing_orb) = &mut self.blowing_orb {
                // Increase blown distance up to a maximum of 120
                blowing_orb.blown_frames += 4;
                if blowing_orb.blown_frames >= 120 {
                    // Can't be blown any further
                    self.blowing_orb = None;
                }
            }
        } else {
            // If we let go of space, we relinquish control over the current orb - it can't be blown any further
            self.blowing_orb = None;
        }

        let resources = storage::get::<Resources>();

        // Set sprite image. If we're currently hurt, the sprite will flash on and off on alternate frames.
        self.image = resources.blank_texture;
        if self.hurt_timer <= 0 || self.hurt_timer % 2 == 1 {
            let dir_index = if self.direction_x > 0 { 1 } else { 0 };
            if self.hurt_timer > 100 {
                if self.health > 0 {
                    self.image = resources.recoil_textures[dir_index];
                } else {
                    let image_i = (game_timer / 4) % 2;
                    self.image = resources.fall_textures[image_i as usize];
                }
            } else if self.fire_timer > 0 {
                self.image = resources.blow_textures[dir_index];
            } else if dx == 0 {
                self.image = resources.still_texture;
            } else {
                let direction_factor = dir_index * 4;
                let image_i = direction_factor + ((game_timer / 8) % 4) as usize;
                self.image = resources.run_textures[image_i];
            }
        }
    }
}

impl Actor for Player {
    fn x(&self) -> i32 {
        self.x
    }

    fn x_mut(&mut self) -> &mut i32 {
        &mut self.x
    }

    fn y(&self) -> i32 {
        self.y
    }

    fn y_mut(&mut self) -> &mut i32 {
        &mut self.y
    }

    fn image(&self) -> Texture2D {
        self.image
    }

    fn anchor(&self) -> Anchor {
        self.anchor
    }
}

impl CollideActor for Player {}

impl GravityActor for Player {
    fn vel_y(&self) -> i32 {
        self.vel_y
    }

    fn vel_y_mut(&mut self) -> &mut i32 {
        &mut self.vel_y
    }

    fn landed(&self) -> bool {
        self.landed
    }

    fn landed_mut(&mut self) -> &mut bool {
        &mut self.landed
    }
}
