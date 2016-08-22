// lib.rs
// Copyright 2016 Alexander Altman
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{fmt, hash, ops, net, io, error, any, cmp, mem};
use std::borrow::BorrowMut;

#[derive(Debug,Hash,PartialEq,Eq,Clone,Copy)]
pub enum InvariantError<E> {
    InvariantFailure,
    OtherError(E),
}

impl<E> Default for InvariantError<E> {
    fn default() -> InvariantError<E> { InvariantError::InvariantFailure }
}

impl<E> Into<Option<E>> for InvariantError<E> {
    fn into(self) -> Option<E> {
        match self {
            InvariantError::InvariantFailure => None,
            InvariantError::OtherError(e) => Some(e),
        }
    }
}

impl<E: fmt::Display> fmt::Display for InvariantError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvariantError::InvariantFailure => write!(formatter, "Invariant failure"),
            InvariantError::OtherError(ref e) => <E as fmt::Display>::fmt(e, formatter),
        }
    }
}

impl<E: error::Error> error::Error for InvariantError<E> {
    fn description(&self) -> &str {
        match *self {
            InvariantError::InvariantFailure => "An invariant failed to hold",
            InvariantError::OtherError(ref e) => {
                <E as error::Error>::description(e)
            },
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            InvariantError::InvariantFailure => None,
            InvariantError::OtherError(ref e) => <E as error::Error>::cause(e),
        }
    }
}

#[derive(Debug,Clone,Copy)]
pub struct Invariant<T: ?Sized, F = Box<FnMut(<T as ToOwned>::Owned) -> bool>>
    where T: ToOwned {
    check: F,
    inner: T,
}

impl<T, F> Invariant<T, F>
    where T: ToOwned {
    pub fn into_inner(self) -> T { self.inner }
}

impl<T: ?Sized, F> Invariant<T, F>
    where T: ToOwned {
    pub fn as_inner_ref(&self) -> &T { &self.inner }
}

impl<T, F> Invariant<T, F>
    where T: ToOwned, F: FnMut(<T as ToOwned>::Owned) -> bool {
    pub fn try_from_inner(inner: T, mut check: F) -> Option<Self> {
        if check(inner.to_owned()) {
            Some(Invariant {
                check: check,
                inner: inner,
            })
        } else {
            None
        }
    }
}

impl<T, F> Invariant<T, F>
    where T: ToOwned, F: FnMut(<T as ToOwned>::Owned) -> bool, <T as ToOwned>::Owned: BorrowMut<T> {
    pub fn with_inner_mut<G, R>(&mut self, op: G) -> Option<R>
        where G: FnOnce(&mut T) -> R {
        self.with_inner_mut_check::<_, R>(|v, _| op(v))
    }

    pub fn with_inner_mut_check<G, R>(&mut self, op: G) -> Option<R>
        where G: FnOnce(&mut T, &mut F) -> R {
        let &mut Invariant { ref mut inner, ref mut check } = self;
        let mut save: T::Owned = inner.to_owned();
        let res: R = op(inner, check);
        if check(inner.to_owned()) {
            Some(res)
        } else {
            mem::swap(inner, save.borrow_mut());
            None
        }
    }
}

impl<T: ?Sized, F> Invariant<T, F>
    where T: ToOwned, T::Owned: Clone, F: ToOwned {
    pub fn to_owned_inner(&self) -> Invariant<T::Owned, F::Owned> {
        Invariant {
            inner: self.as_inner_ref().to_owned(),
            check: self.check.to_owned(),
        }
    }
}

impl<T: ?Sized, F> ops::Deref for Invariant<T, F>
    where T: ToOwned {
    type Target = T;

    fn deref(&self) -> &Self::Target { self.as_inner_ref() }
}

impl<U: ?Sized, T: PartialEq<U> + ?Sized, G, F> PartialEq<Invariant<U, G>> for Invariant<T, F>
    where U: ToOwned, T: ToOwned {
    fn eq(&self, other: &Invariant<U, G>) -> bool {
        <T as PartialEq<U>>::eq(self.as_inner_ref(), other.as_inner_ref())
    }

    fn ne(&self, other: &Invariant<U, G>) -> bool {
        <T as PartialEq<U>>::ne(self.as_inner_ref(), other.as_inner_ref())
    }
}

impl<T: Eq + ?Sized, F> Eq for Invariant<T, F> where T: ToOwned {}

impl<U: ?Sized, T: PartialOrd<U> + ?Sized, G, F> PartialOrd<Invariant<U, G>> for Invariant<T, F>
    where U: ToOwned, T: ToOwned {
    fn partial_cmp(&self, other: &Invariant<U, G>) -> Option<cmp::Ordering> {
        <T as PartialOrd<U>>::partial_cmp(self.as_inner_ref(), other.as_inner_ref())
    }

    fn lt(&self, other: &Invariant<U, G>) -> bool {
        <T as PartialOrd<U>>::lt(self.as_inner_ref(), other.as_inner_ref())
    }

    fn le(&self, other: &Invariant<U, G>) -> bool {
        <T as PartialOrd<U>>::le(self.as_inner_ref(), other.as_inner_ref())
    }

    fn gt(&self, other: &Invariant<U, G>) -> bool {
        <T as PartialOrd<U>>::gt(self.as_inner_ref(), other.as_inner_ref())
    }

    fn ge(&self, other: &Invariant<U, G>) -> bool {
        <T as PartialOrd<U>>::ge(self.as_inner_ref(), other.as_inner_ref())
    }
}

impl<T: Ord + ?Sized, F> Ord for Invariant<T, F>
    where T: ToOwned {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        <T as Ord>::cmp(self.as_inner_ref(), other.as_inner_ref())
    }
}

impl<U, T: AsRef<U>, F> AsRef<U> for Invariant<T, F>
    where T: ToOwned {
    fn as_ref(&self) -> &U { <T as AsRef<U>>::as_ref(self.as_inner_ref()) }
}

impl<T: error::Error + ?Sized, F: fmt::Debug + any::Any> error::Error for Invariant<T, F>
    where T: ToOwned {
    fn description(&self) -> &str { <T as error::Error>::description(self.as_inner_ref()) }

    fn cause(&self) -> Option<&error::Error> { <T as error::Error>::cause(self.as_inner_ref()) }
}

macro_rules! fmt_trait_impl {
    ($t:ident) => {
        impl<T: fmt::$t + ?Sized, F> fmt::$t for Invariant<T, F>
            where T: ToOwned {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                <T as fmt::$t>::fmt(self.as_inner_ref(), formatter)
            }
        }
    }
}

fmt_trait_impl!(Display);
fmt_trait_impl!(LowerExp);
fmt_trait_impl!(UpperExp);
fmt_trait_impl!(Binary);
fmt_trait_impl!(Octal);
fmt_trait_impl!(LowerHex);
fmt_trait_impl!(UpperHex);
fmt_trait_impl!(Pointer);

impl<T: hash::BuildHasher + ?Sized, F> hash::BuildHasher for Invariant<T, F>
    where T: ToOwned {
    type Hasher = T::Hasher;

    fn build_hasher(&self) -> Self::Hasher {
        <T as hash::BuildHasher>::build_hasher(self.as_inner_ref())
    }
}

impl<T: hash::Hash + ?Sized, F> hash::Hash for Invariant<T, F>
    where T: ToOwned {
    fn hash<H>(&self, state: &mut H)
        where H: hash::Hasher {
        <T as hash::Hash>::hash(self.as_inner_ref(), state)
    }
}

impl<T: IntoIterator, F> IntoIterator for Invariant<T, F>
    where T: ToOwned {
    type Item = T::Item;

    type IntoIter = T::IntoIter;

    fn into_iter(self) -> Self::IntoIter { <T as IntoIterator>::into_iter(self.into_inner()) }
}

impl<T: net::ToSocketAddrs, F> net::ToSocketAddrs for Invariant<T, F>
    where T: ToOwned {
    type Iter = T::Iter;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        <T as net::ToSocketAddrs>::to_socket_addrs(self.as_inner_ref())
    }
}

impl<Idx, T: ops::Index<Idx> + ?Sized, F> ops::Index<Idx> for Invariant<T, F>
    where T: ToOwned {
    type Output = T::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        <T as ops::Index<Idx>>::index(self.as_inner_ref(), index)
    }
}

#[cfg(feature = "backtrace")]
mod backtrace_impls;

#[cfg(feature = "backtrace")]
pub use backtrace_impls::*;
