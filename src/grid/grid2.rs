use crate::prelude::*;
use bevy::prelude::*;
use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use super::GridSpec;

/// Represents (row, col) coordinates in the grid.
pub type RowCol = (u16, u16);

/// Extension trait to allow computing distances between RowCols.
pub trait RowColDistance {
    fn distance8(self, other: Self) -> f32;
    fn signed_delta8(self, other: Self) -> Vec2;
}
impl RowColDistance for RowCol {
    /// Distance on a grid with 8-connectivity in cell space.
    fn distance8(self, rowcol2: Self) -> f32 {
        let (row1, col1) = self;
        let (row2, col2) = rowcol2;

        let dx = col2.abs_diff(col1);
        let dy = row2.abs_diff(row1);
        let diagonals = dx.min(dy);
        let straights = dx.max(dy) - diagonals;
        2f32.sqrt() * diagonals as f32 + straights as f32
    }

    /// Signed delta between to rowcol as a float in cell space.
    fn signed_delta8(self, rowcol2: Self) -> Vec2 {
        let (row1, col1) = self;
        let (row2, col2) = rowcol2;
        Vec2 {
            x: (col2 as i16 - col1 as i16) as f32,
            y: (row2 as i16 - row1 as i16) as f32,
        }
    }
}

#[derive(Default)]
pub struct Grid2Plugin<T: Sized + Default>(PhantomData<T>);
impl<T: Sized + Default + Clone + Sync + Send + 'static> Plugin for Grid2Plugin<T> {
    fn build(&self, app: &mut App) {
        app.insert_resource(Grid2::<T>::default()).add_systems(
            FixedUpdate,
            Grid2::<T>::resize_on_change.in_set(SystemStage::PreCompute),
        );
    }
}

/// 2D Grid containing arbitrary data.
#[derive(Clone, Default, Debug, Deref, DerefMut, Resource)]
pub struct Grid2<T: Sized + Default + Clone> {
    #[deref]
    pub spec: GridSpec,
    pub cells: Vec<T>,
}
impl<T: Sized + Default + Clone> Index<RowCol> for Grid2<T> {
    type Output = T;
    fn index(&self, i: RowCol) -> &Self::Output {
        &self.cells[self.flat_index(i)]
    }
}
impl<T: Sized + Default + Clone> IndexMut<RowCol> for Grid2<T> {
    fn index_mut(&mut self, i: RowCol) -> &mut T {
        let flat_i = self.flat_index(i);
        &mut self.cells[flat_i]
    }
}
impl<T: Sized + Default + Clone + Send + Sync + 'static> Grid2<T> {
    pub fn resize_on_change(mut grid: ResMut<Self>, spec: Res<GridSpec>) {
        if spec.is_changed() {
            grid.resize_with(spec.clone());
        }
    }
    /// Resize the grid to match the given spec.
    pub fn resize_with(&mut self, spec: GridSpec) {
        self.spec = spec;
        self.resize();
    }
    /// Resize the grid.
    pub fn resize(&mut self) {
        let num_cells = self.spec.rows as usize * self.spec.cols as usize;
        self.cells.resize(num_cells, T::default());
    }

    pub fn get(&self, rowcol: RowCol) -> Option<&T> {
        let index = self.flat_index(rowcol);
        self.cells.get(index)
    }

    pub fn get_mut(&mut self, rowcol: RowCol) -> Option<&mut T> {
        let index = self.flat_index(rowcol);
        self.cells.get_mut(index)
    }
}
