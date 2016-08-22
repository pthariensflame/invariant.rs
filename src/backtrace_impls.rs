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

use std::{path, os};
use super::*;

extern crate backtrace;

impl<T: backtrace::Frame + ?Sized, F> backtrace::Frame for Invariant<T, F>
    where T: ToOwned {
    fn ip(&self) -> *mut os::raw::c_void { <T as backtrace::Frame>::ip(self.as_inner_ref()) }

    fn symbol_address(&self) -> *mut os::raw::c_void {
        <T as backtrace::Frame>::symbol_address(self.as_inner_ref())
    }
}

impl<T: backtrace::Symbol + ?Sized, F> backtrace::Symbol for Invariant<T, F>
    where T: ToOwned {
    fn name(&self) -> Option<backtrace::SymbolName> {
        <T as backtrace::Symbol>::name(self.as_inner_ref())
    }

    fn addr(&self) -> Option<*mut os::raw::c_void> {
        <T as backtrace::Symbol>::addr(self.as_inner_ref())
    }

    fn filename(&self) -> Option<&path::Path> {
        <T as backtrace::Symbol>::filename(self.as_inner_ref())
    }

    fn lineno(&self) -> Option<u32> { <T as backtrace::Symbol>::lineno(self.as_inner_ref()) }
}
