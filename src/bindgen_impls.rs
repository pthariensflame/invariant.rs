// backtrace_impls.rs
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

use std::fmt;
use super::*;

extern crate bindgen;

impl<T: bindgen::Logger + ?Sized, F: fmt::Debug> bindgen::Logger for Invariant<T, F>
    where T: ToOwned {
    fn error(&self, msg: &str) { <T as bindgen::Logger>::error(self.as_inner_ref(), msg) }

    fn warn(&self, msg: &str) { <T as bindgen::Logger>::warn(self.as_inner_ref(), msg) }
}
