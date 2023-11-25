// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::vec_deque, iter::FusedIterator};

use super::*;

/// A wrapper around either `Iter` or <code>[Rev]\<Iter></code>.
///
/// [Rev]: std::iter::Rev
enum BranchIterator<Iter: Iterator> {
	Normal(Iter),
	Rev(std::iter::Rev<Iter>),
}

/// A borrowing iterator over the children of a [branch].
///
/// This is returned by [`Branch::iter()`].
///
/// [branch]: Branch
pub struct Iter<'branch, Window> {
	iter: BranchIterator<vec_deque::Iter<'branch, Node<Window>>>,
}

/// An owning iterator over the children of a [branch].
///
/// This is returned by [`Branch::into_iter()`].
///
/// [branch]: Branch
pub struct IntoIter<Window> {
	into_iter: BranchIterator<vec_deque::IntoIter<Node<Window>>>,
}

/// A mutably borrowing iterator over the children of a [branch].
///
/// This is returned by [`Branch::iter_mut()`].
///
/// [branch]: Branch
pub struct IterMut<'branch, Window> {
	iter_mut: BranchIterator<vec_deque::IterMut<'branch, Node<Window>>>,
}

macro_rules! impl_iterator {
	(
		for $($Iter:ident<$($lt:lifetime $($mut:ident)?,)? $Window:ident> { $iter:ident }),+$(,)?
		$(; $($tt:tt)*)?
	) => {
		impl_iterator! {
			$(
				for $Iter<$($lt $($mut)?,)? $Window> { $iter } {
					fn into_iter(self) -> Self::IntoIter;
				}
			)+

			$($($tt)*)?
		}
	};

	(
		// $iter is both the iterator field name and the VecDeque iterator method, so those must match
		for $Iter:ident<$($lt:lifetime $($mut:ident)?,)? $Window:ident> { $iter:ident }$(,)? {
			$(#[$inner:meta])*
			fn $into_iter:ident(self) -> Self::$IntoIter:ident;
		}

		$($($tt:tt)+)?
	) => {
		$(impl_iterator! {
			$($tt)+
		})?

		////////////////////////////////////////////////////////////////////////////////////////////
		// Iterator impls
		////////////////////////////////////////////////////////////////////////////////////////////

		impl<$($lt,)? $Window> Iterator for $Iter<$($lt,)? $Window> {
			type Item = $(&$lt $($mut)?)? Node<$Window>;

			#[inline]
			fn next(&mut self) -> Option<Self::Item> {
				match &mut self.$iter {
					BranchIterator::Normal($iter) => $iter.next(),
					BranchIterator::Rev($iter) => $iter.next(),
				}
			}

			#[inline]
			fn size_hint(&self) -> (usize, Option<usize>) {
				match &self.$iter {
					BranchIterator::Normal($iter) => $iter.size_hint(),
					BranchIterator::Rev($iter) => $iter.size_hint(),
				}
			}

			#[inline]
			fn fold<Acc, F>(self, accum: Acc, f: F) -> Acc
			where
				F: FnMut(Acc, Self::Item) -> Acc,
			{
				match self.$iter {
					BranchIterator::Normal($iter) => $iter.fold(accum, f),
					BranchIterator::Rev($iter) => $iter.fold(accum, f),
				}
			}

			#[inline]
			fn last(self) -> Option<Self::Item> {
				match self.$iter {
					BranchIterator::Normal($iter) => $iter.last(),
					BranchIterator::Rev($iter) => $iter.last(),
				}
			}
		}

		impl<$($lt,)? $Window> DoubleEndedIterator for $Iter<$($lt,)? $Window> {
			#[inline]
			fn next_back(&mut self) -> Option<Self::Item> {
				match &mut self.$iter {
					BranchIterator::Normal($iter) => $iter.next_back(),
					BranchIterator::Rev($iter) => $iter.next_back(),
				}
			}

			#[inline]
			fn rfold<Acc, F>(self, accum: Acc, f: F) -> Acc
			where
				F: FnMut(Acc, Self::Item) -> Acc,
			{
				match self.$iter {
					BranchIterator::Normal($iter) => $iter.rfold(accum, f),
					BranchIterator::Rev($iter) => $iter.rfold(accum, f),
				}
			}
		}

		impl<$($lt,)? $Window> ExactSizeIterator for $Iter<$($lt,)? $Window> {
			#[inline]
			fn len(&self) -> usize {
				match &self.$iter {
					BranchIterator::Normal($iter) => $iter.len(),
					BranchIterator::Rev($iter) => $iter.len(),
				}
			}
		}

		impl<$($lt,)? $Window> FusedIterator for $Iter<$($lt,)? $Window> {}
	};
}

impl_iterator! {
	for Iter<'branch, Window> { iter };
	for IntoIter<Window> { into_iter };
	for IterMut<'branch mut, Window> { iter_mut };
}

impl<'branch, Window> IntoIterator for &'branch Branch<Window> {
	type Item = &'branch Node<Window>;
	type IntoIter = Iter<'branch, Window>;

	fn into_iter(self) -> Self::IntoIter {
		let borrow = RefCell::borrow(&self.0);

		Iter {
			iter: if self.orientation().reversed() {
				BranchIterator::Rev(borrow.children.iter().rev())
			} else {
				BranchIterator::Normal(borrow.children.iter())
			},
		}
	}
}

impl<'branch, Window> IntoIterator for &'branch mut Branch<Window> {
	type Item = &'branch mut Node<Window>;
	type IntoIter = IterMut<'branch, Window>;

	fn into_iter(self) -> Self::IntoIter {
		let borrow = RefCell::borrow_mut(&self.0);

		IterMut {
			iter_mut: if self.orientation().reversed() {
				BranchIterator::Rev(borrow.children.iter_mut().rev())
			} else {
				BranchIterator::Normal(borrow.children.iter_mut())
			},
		}
	}
}

impl<Window> Branch<Window> {
	/// Returns a borrowing iterator over the direct children of this branch.
	pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
		self.into_iter()
	}

	/// Returns an owning iterator over the direct children of this branch.
	pub fn into_iter(self) -> Result<IntoIter<Window>, NodeUnwrapError> {
		let orientation = self.orientation();

		match Rc::try_unwrap(self.0) {
			Ok(ref_cell) => Ok(IntoIter {
				into_iter: if orientation.reversed() {
					BranchIterator::Rev(ref_cell.into_inner().children.into_iter().rev())
				} else {
					BranchIterator::Normal(ref_cell.into_inner().children.into_iter())
				},
			}),

			Err(_) => Err(NodeUnwrapError),
		}
	}

	/// Returns a mutably borrowing iterator over the direct children of this branch.
	pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
		self.into_iter()
	}
}
