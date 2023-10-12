use super::*;

#[derive(Clone)]
pub struct Heightmap {
    pub data: Array2D<f32>,
    pub offset: vec2<i32>,
}

impl Heightmap {
    pub fn new() -> Self {
        Self {
            data: Array2D::filled_with(0.0, 0, 0),
            offset: vec2::ZERO,
        }
    }

    pub fn bounding_box(&self) -> Option<Aabb2<i32>> {
        Some(
            Aabb2::point(self.offset).extend_positive(
                vec2(self.data.num_rows(), self.data.num_columns()).map(|x| x as i32),
            ),
        )
    }

    pub fn ensure_bounds(&mut self, bb: Aabb2<i32>) {
        let old_bb = self.bounding_box().unwrap_or(Aabb2::ZERO);
        let new_bb = Aabb2 {
            min: old_bb.min.zip(bb.min).map(|(a, b)| i32::min(a, b)),
            max: old_bb.max.zip(bb.max).map(|(a, b)| i32::max(a, b)),
        };
        if new_bb != old_bb {
            let old_data = std::mem::replace(
                &mut self.data,
                Array2D::filled_with(0.0, new_bb.width() as usize, new_bb.height() as usize),
            );
            let offset = (old_bb.bottom_left() - new_bb.bottom_left()).map(|x| x as usize);
            for (row_idx, row) in old_data.rows_iter().enumerate() {
                for (col_idx, &height) in row.enumerate() {
                    self.data
                        .set(row_idx + offset.x, col_idx + offset.y, height)
                        .unwrap();
                }
            }
            self.offset = new_bb.bottom_left();
        }
    }

    pub fn get(&self, p: vec2<i32>) -> Option<f32> {
        let p = p - self.offset;
        if p.x < 0 || p.y < 0 {
            return None;
        }
        let p = p.map(|x| x as usize);
        self.data.get(p.x, p.y).copied()
    }

    pub fn get_mut(&mut self, p: vec2<i32>) -> Option<&mut f32> {
        let p = p - self.offset;
        if p.x < 0 || p.y < 0 {
            return None;
        }
        let p = p.map(|x| x as usize);
        self.data.get_mut(p.x, p.y)
    }
}

impl Default for Heightmap {
    fn default() -> Self {
        Self::new()
    }
}
