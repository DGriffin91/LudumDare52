use bevy::{math::*, prelude::*};
use pathfinding::prelude::astar;

#[derive(Resource)]
pub struct GameBoard {
    pub size: [usize; 2],
    pub position: IVec2,
    pub board: Vec<Option<Entity>>,
    pub has_blobby: Vec<bool>,
    pub start: IVec2,
    pub dest: IVec2,
}

impl Default for GameBoard {
    fn default() -> Self {
        GameBoard::new(ivec2(-12, -12), [24, 24], ivec2(0, 0), ivec2(22, 22))
    }
}

impl GameBoard {
    pub fn new(position: IVec2, size: [usize; 2], start: IVec2, dest: IVec2) -> GameBoard {
        let board = vec![None; size[0] * size[1]];
        GameBoard {
            size,
            position,
            has_blobby: vec![false; size[0] * size[1]],
            board,
            start,
            dest,
        }
    }

    pub fn reset_has_blobby(&mut self) {
        self.has_blobby = vec![false; self.size[0] * self.size[1]];
    }

    #[inline(always)]
    pub fn ls_to_ws(&self, ls: IVec2) -> IVec2 {
        ls + self.position
    }

    #[inline(always)]
    pub fn ls_to_ws_f(&self, ls: Vec2) -> Vec2 {
        ls + vec2(self.position.x as f32, self.position.y as f32)
    }

    #[inline(always)]
    pub fn ws_to_ls(&self, ws: IVec2) -> IVec2 {
        ws - self.position
    }

    #[inline(always)]
    pub fn ws_to_ls_f(&self, ws: Vec2) -> Vec2 {
        ws - vec2(self.position.x as f32, self.position.y as f32)
    }

    #[inline(always)]
    pub fn ls_to_idx(&self, ls: IVec2) -> usize {
        let x = (ls.x as usize).clamp(0, self.size[0] - 1);
        let y = (ls.y as usize).clamp(0, self.size[1] - 1);
        x + y * self.size[1]
    }

    #[inline(always)]
    pub fn idx_to_ls(&self, idx: usize) -> IVec2 {
        let x = idx % self.size[0];
        let y = (idx / self.size[0]) % self.size[1];

        ivec2(x as i32, y as i32)
    }

    pub fn path(&self, start: IVec2, end: IVec2) -> Option<(Vec<IVec2>, u32)> {
        astar(
            &start,
            |p| self.successors(*p),
            |p| {
                let a = (end - *p).abs();
                (a.x + a.y) as u32
            },
            |p| *p == end,
        )
    }

    #[inline(always)]
    pub fn successors(&self, ls: IVec2) -> Vec<(IVec2, u32)> {
        let mut s = Vec::new();
        for offset in [
            //ivec2(0, 0),
            ivec2(-1, 0),
            ivec2(1, 0),
            ivec2(0, -1),
            ivec2(0, 1),
            ivec2(-1, -1),
            ivec2(-1, 1),
            ivec2(1, -1),
            ivec2(1, 1),
        ] {
            let potential_pos = ls + offset;
            if potential_pos.clamp(IVec2::ZERO, ivec2(self.size[0] as i32, self.size[1] as i32))
                != potential_pos
                || self.board[self.ls_to_idx(potential_pos)].is_some()
            {
                continue;
            } else {
                s.push((potential_pos, 1));
            }
        }

        s
    }
    pub fn ws_vec3_to_ls(&self, ws: Vec3) -> IVec2 {
        self.ws_to_ls(vec3_to_ivec2(ws))
    }
    pub fn ws_vec3_to_ls_f(&self, ws: Vec3) -> Vec2 {
        self.ws_to_ls_f(vec3_to_vec2(ws))
    }
    pub fn ls_to_ws_vec3(&self, ls: IVec2) -> Vec3 {
        ivec2_to_vec3(self.ls_to_ws(ls)) + vec3(0.5, 0.0, 0.5)
    }
    pub fn ls_f_to_ws_vec3(&self, ls: Vec2) -> Vec3 {
        vec2_to_vec3(self.ls_to_ws_f(ls)) + vec3(0.5, 0.0, 0.5)
    }
    pub fn destroy(&mut self, com: &mut Commands, idx: usize) {
        if let Some(entity) = &self.board[idx] {
            com.entity(*entity).despawn_recursive();
        }
        self.board[idx] = None;
    }
}

pub fn vec3_to_ivec2(p: Vec3) -> IVec2 {
    ivec2(p.x.floor() as i32, p.z.floor() as i32)
}

pub fn vec3_to_vec2(p: Vec3) -> Vec2 {
    vec2(p.x, p.z)
}

pub fn ivec2_to_vec3(p: IVec2) -> Vec3 {
    vec3(p.x as f32, 0.0, p.y as f32)
}

pub fn vec2_to_vec3(p: Vec2) -> Vec3 {
    vec3(p.x, 0.0, p.y)
}
