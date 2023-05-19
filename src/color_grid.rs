use cursive::{
    theme::{ColorStyle},
    Vec2,
};
use std::ops::Index;
use crate::block::{Block, BlockWithPos};
use crate::lrd::{ LR, LRD };

pub struct ColorGrid {
    pub block: BlockWithPos,
    colors: Vec<Vec<ColorStyle>>,
    background_color: (ColorStyle, ColorStyle),
    pub warning_color: ColorStyle,
}

impl ColorGrid {
    pub fn new(background_color: (ColorStyle, ColorStyle), warning_color: ColorStyle, width: usize, height: usize) -> Self {
        let mut colors = Vec::with_capacity(height);
        for h in 0..height {
            let mut row = Vec::with_capacity(width);
            for w in 0..width {
                    let color = if (w + h) % 2 == 0 {
                        background_color.0
                    } else {
                        background_color.1
                    };
                    row.push(color);
            }
            colors.push(row);
        }
        Self {
            block: Self::insert_new_block(Block::default(), width, height),
            colors,
            background_color,
            warning_color,
        }
    }

    pub fn insert(&mut self, block: Block) {
        self.block = ColorGrid::insert_new_block(block, self.width(), self.height());
    }

    fn insert_new_block(block: Block, width: usize, height: usize) -> BlockWithPos {
        ColorGrid::fit(block, Vec2::new(width / 2, 0), width, height).unwrap()
    }

    fn fit(block: Block, _pos: Vec2, width: usize, height: usize) -> Option<BlockWithPos> {
        let mut pos = _pos;
        for _ in 0..6 {
            let mut possible = true;
            for cell in block.cells() {
                let x = pos.x as i32 + cell.0;
                let y = pos.y as i32 + cell.1;
                if x < 0 {
                    pos.x += 1;
                    possible = false;
                    break;
                } else if x >= width as i32 {
                    pos.x -= 1;
                    possible = false;
                    break;
                } else if y < 0 {
                    pos.y += 1;
                    possible = false;
                    break;
                } else if y >= height as i32{
                    pos.y -= 1;
                    possible = false;
                    break;
                }
            }
            if possible {
                return Some(BlockWithPos::new(block, pos))
            }
        }
        None
    }

    pub fn move_block_lrd(&self, block: &BlockWithPos, lrd: LRD) -> (Option<BlockWithPos>, bool) {
        let (can_move, stop) = self.can_move(block, &lrd);
        if !can_move {
            return (None, stop)
        }
        let delta = lrd.delta();
        let x = block.pos.x as i32 + delta.0;
        let y = block.pos.y as i32 + delta.1;
        let bwp = BlockWithPos::new(block.block.clone(), Vec2::new(x as usize, y as usize));
        (Some(bwp), stop)
    }

    fn can_move(&self, block: &BlockWithPos, lrd: &LRD) -> (bool, bool) {
        let delta = lrd.delta();
        let mut stop = false;
        let board_width = self.width() as i32;
        let board_height = self.height() as i32;
        for cell in block.cells() {
            let next_x = cell.x as i32 + delta.0;
            let next_y =  cell.y as i32 + delta.1;
            if next_x < 0 || next_x >= board_width || next_y < 0 || next_y >= board_height || self.is_occupied(next_x as usize, next_y as usize)
            {
                return (false, false);
            }
            if next_y + 1 == board_height || self.is_occupied(next_x as usize, next_y as usize + 1)
            {
                stop = true;
            }
        }
        (true, stop)
    }

    pub fn hint(&self) -> BlockWithPos {
        let mut hint = self.block.clone();
        let mut stopped = false;
        while !stopped {
            let (block, hit_bottom) = self.move_block_lrd(&hint, LRD::Down);
            stopped = hit_bottom || block.is_none();
            hint = block.unwrap_or(hint);
        }
        hint
    }

    pub fn on_down(&mut self, is_drop: bool) -> (bool, bool, usize) {
        let mut stopped = false;
        let mut hit_bottom = is_drop;
        let mut current: Option<BlockWithPos>;
        while !stopped {
            (current, hit_bottom)= self.move_block_lrd(&self.block, LRD::Down);
            match current {
                Some(b) => self.block = b,
                None => return (true, true, 0),
            }

            stopped = hit_bottom || !is_drop;
        }
        let score = if hit_bottom {
            self.merge_block()
        } else {
            0
        };
        (false, hit_bottom, score)
    }

    pub fn handle_lr(&mut self, lr: LR) {
        let (block, _) = self.move_block_lrd(&self.block, lr.to_lrd());
        if block.is_some() {
            self.block = block.unwrap();
        }
    }

    pub fn rotate(&mut self) {
        let next_block = self.block.block.rotate();
        ColorGrid::fit(next_block, self.block.pos, self.width(), self.height())
            .map(|b| self.block = b);
    }

    pub fn width(&self) -> usize {
        self.colors[0].len()
    }

    pub fn height(&self) -> usize {
        self.colors.len()
    }

    fn set_background(&mut self, x: usize, y: usize) {
        let color = if (x + y) % 2 == 0 {
            self.background_color.0
        } else {
            self.background_color.1
        };
        self.colors[y][x] = color;
    }

    pub fn renew(&mut self) {
        for x in 0..self.width() {
            for y in 0..self.height() {
                self.set_background(x, y);
            }
        }
        self.insert(Block::default());
    }

    pub fn is_occupied(&self, x: usize, y: usize) -> bool {
        self.colors[y][x] != self.background_color.0 && 
        self.colors[y][x] != self.background_color.1 &&
        self.colors[y][x] != self.warning_color
    }

    pub fn merge_block(&mut self) -> usize {
        self.fill_board_with_block();
        self.remove_rows_if_possible()
    }

    fn fill_board_with_block(&mut self) {
        for cell in self.block.cells() {
            self.colors[cell.y][cell.x] = self.block.color();
        }
    }

    fn remove_rows_if_possible(&mut self) -> usize {
        let mut rows_to_remove = Vec::new();
        for _y in 0..self.height() {
            let y = self.height() - _y - 1;
            let mut remove = true;
            for x in 0..self.width() {
                if !self.is_occupied(x, y) {
                    remove = false;
                    break;
                }
            }
            if remove {
                rows_to_remove.push(y);
            }
        }
        let score = rows_to_remove.len();
        self.remove_rows(rows_to_remove);
        score
    }

    fn remove_rows(&mut self, rows_to_remove: Vec<usize>) {
        if rows_to_remove.is_empty() {
            return;
        }
        let mut fill_y = self.height() - 1;
        let mut check_y = self.height() - 1;
        for row in rows_to_remove {
            while check_y > row {
                if fill_y != check_y {
                    self.colors[fill_y] = self.background_row(check_y, fill_y);
                }
                fill_y -= 1;
                check_y -= 1;
            }
            check_y = row - 1;
        }
        while check_y > 0 {
            self.colors[fill_y] = self.background_row(check_y, fill_y);
            fill_y -= 1;
            check_y -= 1;
        }
        while fill_y > 0 {
            for x in 0..self.width() {
                self.set_background(x, fill_y);
            }
            fill_y -= 1;
        }
    }

    fn background_row(&self, from: usize, to: usize) -> Vec<ColorStyle> {
        let mut row = Vec::new();
        for w in 0..self.width() {
            if self.is_occupied(w, from) {
                row.push(self.colors[from][w]);
                continue
            }
            let color = if (w + to) % 2 == 0 {
                self.background_color.0
            } else {
                self.background_color.1
            };
            row.push(color);        
        }
        row
    }
}

impl Index<usize> for ColorGrid {
    type Output = Vec<ColorStyle>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.colors[index]
    }
}