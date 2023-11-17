// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{collections::vec_deque, iter::FusedIterator};

use super::*;

/// A wrapper around either `Iter` or <code>[Rev]\<Iter></code>.
///
/// [Rev]: std::iter::Rev
enum GroupIterator<Iter: Iterator> {
	Normal(Iter),
	Rev(std::iter::Rev<Iter>),
}

/// A borrowing iterator over the children of a [group].
///
/// This is returned by [`GroupNode::iter()`].
///
/// [group]: GroupNode
pub struct Iter<'group, Window> {
	iter: GroupIterator<vec_deque::Iter<'group, Node<Window>>>,
}

/// An owning iterator over the children of a [group].
///
/// This is returned by [`GroupNode::into_iter()`].
///
/// [group]: GroupNode
pub struct IntoIter<Window> {
	into_iter: GroupIterator<vec_deque::IntoIter<Node<Window>>>,
}

/// A mutably borrowing iterator over the children of a [group].
///
/// This is returned by [`GroupNode::iter_mut()`].
///
/// [group]: GroupNode
pub struct IterMut<'group, Window> {
	iter_mut: GroupIterator<vec_deque::IterMut<'group, Node<Window>>>,
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
		// $Iter impls
		////////////////////////////////////////////////////////////////////////////////////////////

		impl<$($lt,)? $Window> $Iter<$($lt,)? $Window> {
			fn new(group: $(&$lt $($mut)?)? GroupNode<$Window>) -> Self {
				Self {
					$iter: if !group.orientation().reversed() {
						GroupIterator::Normal(group.children.$iter())
					} else {
						GroupIterator::Rev(group.children.$iter().rev())
					},
				}
			}
		}

		impl<$($lt,)? $Window> IntoIterator for $(&$lt $($mut)?)? GroupNode<$Window> {
			type Item = $(&$lt $($mut)?)? Node<$Window>;
			type $IntoIter = $Iter<$($lt,)? $Window>;

			$(#[$inner])*
			fn $into_iter(self) -> Self::$IntoIter {
				$Iter::new(self)
			}
		}

		////////////////////////////////////////////////////////////////////////////////////////////
		// Iterator impls
		////////////////////////////////////////////////////////////////////////////////////////////

		impl<$($lt,)? $Window> Iterator for $Iter<$($lt,)? $Window> {
			type Item = $(&$lt $($mut)?)? Node<$Window>;

			#[inline]
			fn next(&mut self) -> Option<Self::Item> {
				match &mut self.$iter {
					GroupIterator::Normal($iter) => $iter.next(),
					GroupIterator::Rev($iter) => $iter.next(),
				}
			}

			#[inline]
			fn size_hint(&self) -> (usize, Option<usize>) {
				match &self.$iter {
					GroupIterator::Normal($iter) => $iter.size_hint(),
					GroupIterator::Rev($iter) => $iter.size_hint(),
				}
			}

			#[inline]
			fn fold<Acc, F>(self, accum: Acc, f: F) -> Acc
			where
				F: FnMut(Acc, Self::Item) -> Acc,
			{
				match self.$iter {
					GroupIterator::Normal($iter) => $iter.fold(accum, f),
					GroupIterator::Rev($iter) => $iter.fold(accum, f),
				}
			}

			#[inline]
			fn last(self) -> Option<Self::Item> {
				match self.$iter {
					GroupIterator::Normal($iter) => $iter.last(),
					GroupIterator::Rev($iter) => $iter.last(),
				}
			}
		}

		impl<$($lt,)? $Window> DoubleEndedIterator for $Iter<$($lt,)? $Window> {
			#[inline]
			fn next_back(&mut self) -> Option<Self::Item> {
				match &mut self.$iter {
					GroupIterator::Normal($iter) => $iter.next_back(),
					GroupIterator::Rev($iter) => $iter.next_back(),
				}
			}

			#[inline]
			fn rfold<Acc, F>(self, accum: Acc, f: F) -> Acc
			where
				F: FnMut(Acc, Self::Item) -> Acc,
			{
				match self.$iter {
					GroupIterator::Normal($iter) => $iter.rfold(accum, f),
					GroupIterator::Rev($iter) => $iter.rfold(accum, f),
				}
			}
		}

		impl<$($lt,)? $Window> ExactSizeIterator for $Iter<$($lt,)? $Window> {
			#[inline]
			fn len(&self) -> usize {
				match &self.$iter {
					GroupIterator::Normal($iter) => $iter.len(),
					GroupIterator::Rev($iter) => $iter.len(),
				}
			}
		}

		impl<$($lt,)? $Window> FusedIterator for $Iter<$($lt,)? $Window> {}
	};
}

impl_iterator! {
	for Iter<'group, Window> { iter };

	for IntoIter<Window> { into_iter } {
		/// Returns an owning iterator over the direct children of this group.
		fn into_iter(self) -> Self::IntoIter;
	}

	for IterMut<'group mut, Window> { iter_mut };
}

impl<Window> GroupNode<Window> {
	/// Returns a borrowing iterator over the direct children of this group.
	pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
		self.into_iter()
	}

	/// Returns a mutably borrowing iterator over the direct children of this group.
	pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
		self.into_iter()
	}
}
