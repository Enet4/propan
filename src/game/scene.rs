//! Module for the `Scene` type, which represents a collection of walls and
//! other static props. This provides a single static map collision
//! handling point, thus enabling certain optimizations.

use na::Vector2;
use physics::Positioned;
use std::collections::HashMap;

pub type Scene<P> = FlatScene<P>;
pub type CellId = (i32, i32);

/// A naive scene in which all props are exhaustively traversed.
#[derive(Debug)]
pub struct FlatScene<P> {
    props: Vec<P>,
}

impl<P> FlatScene<P>
where
    P: Positioned,
{
    pub fn from_objects<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = P>,
    {
        FlatScene { props: iter.into_iter().collect() }
    }

    /// obtain an iterator to all objects, regardless of the given position
    #[inline]
    pub fn at(&self, _: Vector2<f32>) -> impl Iterator<Item = &P> {
        self.props.iter()
    }

    /// obtain a mutable iterator to all objects around the given position
    /// (on the same cell and surrounding cells)
    #[inline]
    pub fn at_mut(&mut self, _: Vector2<f32>) -> impl Iterator<Item = &mut P> {
        self.props.iter_mut()
    }
}

impl<'a, P> IntoIterator for &'a FlatScene<P>
{
    type IntoIter = ::std::slice::Iter<'a, P>;
    type Item = &'a P;

    fn into_iter(self) -> Self::IntoIter {
        self.props.iter()
    }
}

impl<'a, P> IntoIterator for &'a mut FlatScene<P>
{
    type IntoIter = ::std::slice::IterMut<'a, P>;
    type Item = &'a mut P;

    fn into_iter(self) -> Self::IntoIter {
        self.props.iter_mut()
    }
}

/// A scene in which props are kept in a hash-indexed grid of wall vectors.
#[derive(Debug)]
pub struct HashGridScene<W> {
    cell_size: f32,
    grid: HashMap<CellId, Vec<W>>,
}

impl<W> HashGridScene<W>
where
    W: Positioned,
{
    pub fn from_objects<I>(cell_size: f32, iter: I) -> Self
    where
        I: IntoIterator<Item = W>,
    {
        let mut grid = HashMap::<_, Vec<_>>::new();

        for w in iter {
            let cell = position_to_cell(w.position(), cell_size);
            grid.entry(cell).or_default().push(w);
        }

        HashGridScene { cell_size, grid }
    }

    /// obtain an iterator to all objects around the given position
    /// (on the same cell and surrounding cells)
    pub fn at(&self, pos: Vector2<f32>) -> impl Iterator<Item = &W> {
        let cell = self.to_cell(pos);
        [-1, 0, -1]
            .iter()
            .cloned()
            .flat_map(|x| [-1, 0, 1].iter().map(move |y| (x, *y)))
            .map(move |(x, y)| (x + cell.0, y + cell.1))
            .flat_map(move |cell| self.at_single_cell(cell))
    }

    /// obtain a mutable iterator to all objects around the given position
    /// (on the same cell and surrounding cells)
    pub fn at_mut<'a>(&'a mut self, pos: Vector2<f32>) -> impl Iterator<Item = &'a mut W> {
        let cell = self.to_cell(pos);
        [-1, 0, -1]
            .iter()
            .cloned()
            .flat_map(|x| [-1, 0, 1].iter().map(move |y| (x, *y)))
            .map(move |(x, y)| (x + cell.0, y + cell.1))
            .flat_map(move |cell| {
                // !!! I have joined the dark side
                unsafe {
                    let p: *mut Self = self;
                    (&mut *p).at_single_cell_mut(cell)
                }
            })
    }

    /// obtain an iterator to all objects in the same cell
    fn at_single_cell(&self, cell: CellId) -> impl Iterator<Item = &W> {
        self.grid.get(&cell).into_iter().flatten()
    }

    /// obtain a mutable iterator to all objects in the same cell
    fn at_single_cell_mut<'a>(&'a mut self, cell: CellId) -> impl Iterator<Item = &'a mut W> {
        self.grid.get_mut(&cell).into_iter().flatten()
    }

    fn to_cell(&self, pos: Vector2<f32>) -> CellId {
        position_to_cell(pos, self.cell_size)
    }
}

#[derive(Debug)]
pub struct CellIter<'a, W: 'a, I> {
    cells: I,
    cell_values: ::std::slice::Iter<'a, W>,
}

impl<'a, W: 'a, I> Iterator for CellIter<'a, W, I>
where
    I: Iterator<Item = &'a Vec<W>>,
{
    type Item = &'a W;

    fn next(&mut self) -> Option<Self::Item> {
        {
            let w = self.cell_values.next();
            if w.is_some() {
                return w;
            }
        }
        match self.cells.next() {
            None => None,
            Some(next_cell_values) => {
                self.cell_values = next_cell_values.into_iter();
                self.next()
            }
        }
    }
}

#[derive(Debug)]
pub struct CellIterMut<'a, W: 'a, I> {
    cells: I,
    cell_values: ::std::slice::IterMut<'a, W>,
}

impl<'a, W: 'a, I> Iterator for CellIterMut<'a, W, I>
where
    I: Iterator<Item = &'a mut Vec<W>>,
{
    type Item = &'a mut W;

    fn next(&mut self) -> Option<Self::Item> {
        {
            let w = self.cell_values.next();
            if w.is_some() {
                return w;
            }
        }
        match self.cells.next() {
            None => None,
            Some(next_cell_values) => {
                self.cell_values = next_cell_values.into_iter();
                self.next()
            }
        }
    }
}

impl<'a, W> IntoIterator for &'a HashGridScene<W> {
    type IntoIter = CellIter<'a, W, ::std::collections::hash_map::Values<'a, CellId, Vec<W>>>;
    type Item = &'a W;

    fn into_iter(self) -> Self::IntoIter {
        let mut cells = self.grid.values();
        let cell_values = cells.next().map(|v| v.into_iter()).unwrap_or([].iter());
        CellIter { cells, cell_values }
    }
}

impl<'a, W> IntoIterator for &'a mut HashGridScene<W> {
    type IntoIter = CellIterMut<'a, W, ::std::collections::hash_map::ValuesMut<'a, CellId, Vec<W>>>;
    type Item = &'a mut W;

    fn into_iter(self) -> Self::IntoIter {
        let mut cells = self.grid.values_mut();
        let cell_values = cells.next().map(|v| v.into_iter()).unwrap_or([].iter_mut());
        CellIterMut { cells, cell_values }
    }
}

#[inline]
fn position_to_cell(pos: Vector2<f32>, cell_size: f32) -> CellId {
    ((pos[0] / cell_size) as i32, (pos[1] / cell_size) as i32)
}
