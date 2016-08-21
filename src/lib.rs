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

use std::*;

#[derive(Debug,Clone,Copy)]
pub struct Invariant<T: ?Sized, F = Box<FnMut(<T as ToOwned>::Owned) -> bool>>
    where T: ToOwned, F: FnMut(<T as ToOwned>::Owned) -> bool {
    check: F,
    inner: T,
}

impl<T, F> Invariant<T, F>
    where T: ToOwned, F: FnMut(<T as ToOwned>::Owned) -> bool {
    pub fn into_inner(self) -> T { self.inner }

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

impl<T: ?Sized, F> Invariant<T, F>
    where T: ToOwned, F: FnMut(<T as ToOwned>::Owned) -> bool {
    pub fn as_inner_ref(&self) -> &T { &self.inner }
}

impl<T: ?Sized, F> ops::Deref for Invariant<T, F>
    where T: ToOwned, F: FnMut(<T as ToOwned>::Owned) -> bool {
    type Target = T;

    fn deref(&self) -> &Self::Target { self.as_inner_ref() }
}

impl<T: fmt::Display + ?Sized, F> fmt::Display for Invariant<T, F>
    where T: ToOwned, F: FnMut(<T as ToOwned>::Owned) -> bool {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        <T as fmt::Display>::fmt(self.as_inner_ref(), formatter)
    }
}

impl<T: hash::Hash + ?Sized, F> hash::Hash for Invariant<T, F>
    where T: ToOwned, F: FnMut(<T as ToOwned>::Owned) -> bool {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        <T as hash::Hash>::hash(self.as_inner_ref(), state)
    }
}
