//! Iterator types for [`Dataset`](super::Dataset).
//!
//! This module provides named iterator types for iterating over [`Column`] items.

use super::Column;

/// An iterator over references to columns in a dataset.
///
/// Created by [`Dataset::iter`](super::Dataset::iter).
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    inner: std::slice::Iter<'a, Column>,
}

impl<'a> Iter<'a> {
    pub(super) fn new(columns: &'a [Column]) -> Self {
        Self {
            inner: columns.iter(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Column;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

/// A mutable iterator over columns in a dataset.
///
/// Created by [`Dataset::iter_mut`](super::Dataset::iter_mut).
#[derive(Debug)]
pub struct IterMut<'a> {
    inner: std::slice::IterMut<'a, Column>,
}

impl<'a> IterMut<'a> {
    pub(super) fn new(columns: &'a mut [Column]) -> Self {
        Self {
            inner: columns.iter_mut(),
        }
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Column;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> ExactSizeIterator for IterMut<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> DoubleEndedIterator for IterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

/// An owning iterator over columns in a dataset.
///
/// Created by calling `into_iter()` on a [`Dataset`](super::Dataset).
#[derive(Debug)]
pub struct IntoIter {
    inner: std::vec::IntoIter<Column>,
}

impl IntoIter {
    pub(super) fn new(columns: Vec<Column>) -> Self {
        Self {
            inner: columns.into_iter(),
        }
    }
}

impl Iterator for IntoIter {
    type Item = Column;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

/// An iterator over column names in a dataset.
///
/// Created by [`Dataset::column_names`](super::Dataset::column_names).
#[derive(Debug, Clone)]
pub struct ColumnNames<'a> {
    inner: Iter<'a>,
}

impl<'a> ColumnNames<'a> {
    pub(super) fn new(columns: &'a [Column]) -> Self {
        Self {
            inner: Iter::new(columns),
        }
    }
}

impl<'a> Iterator for ColumnNames<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Column::name)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a> ExactSizeIterator for ColumnNames<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> DoubleEndedIterator for ColumnNames<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(Column::name)
    }
}
