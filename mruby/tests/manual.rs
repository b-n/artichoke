#[macro_use]
extern crate log;

use mruby::convert::TryFromMrb;
use mruby::def::{ClassLike, Define};
use mruby::file::MrbFile;
use mruby::interpreter::{Interpreter, Mrb};
use mruby::sys;
use mruby::value::Value;
use mruby::{interpreter_or_raise, unwrap_or_raise};
use std::cell::RefCell;
use std::ffi::{c_void, CString};
use std::mem;
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
struct Container {
    inner: i64,
}

impl MrbFile for Container {
    fn require(interp: Mrb) {
        extern "C" fn free(_mrb: *mut sys::mrb_state, data: *mut c_void) {
            unsafe {
                debug!("preparing to free Container instance");
                // Implictly dropped by going out of scope
                let inner = mem::transmute::<*mut c_void, Rc<RefCell<Container>>>(data);
                debug!(
                    "freeing Container instance with value: {}",
                    inner.borrow().inner
                );
            }
        }

        extern "C" fn initialize(
            mrb: *mut sys::mrb_state,
            mut slf: sys::mrb_value,
        ) -> sys::mrb_value {
            unsafe {
                let interp = interpreter_or_raise!(mrb);
                let api = interp.borrow();

                let int = mem::uninitialized::<sys::mrb_int>();
                let argspec = CString::new(sys::specifiers::INTEGER).expect("argspec");
                sys::mrb_get_args(mrb, argspec.as_ptr(), &int);
                let cont = Container { inner: int };
                let data = Rc::new(RefCell::new(cont));
                debug!("Storing `Container` refcell in self instance: {:?}", data);
                let ptr = mem::transmute::<Rc<RefCell<Container>>, *mut c_void>(data);

                debug!(
                    "interpreter strong ref count = {}",
                    Rc::strong_count(&interp)
                );
                let spec = api.class_spec::<Container>();
                sys::mrb_sys_data_init(&mut slf, ptr, spec.data_type());

                slf
            }
        }

        extern "C" fn value(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
            unsafe {
                let interp = interpreter_or_raise!(mrb);
                let api = interp.borrow();
                let spec = api.class_spec::<Container>();

                debug!("pulled mrb_data_type from user data with class: {:?}", spec);
                let ptr = sys::mrb_data_get_ptr(mrb, slf, spec.data_type());
                let data = mem::transmute::<*mut c_void, Rc<RefCell<Container>>>(ptr);
                let clone = Rc::clone(&data);
                let cont = clone.borrow();

                let value = unwrap_or_raise!(interp, Value::try_from_mrb(&interp, cont.inner));
                mem::forget(data);
                value
            }
        }

        {
            let mut api = interp.borrow_mut();
            api.def_class::<Container>("Container", None, Some(free));
            let spec = api.class_spec_mut::<Self>();
            spec.add_method("initialize", initialize, sys::mrb_args_req(1));
            spec.add_method("value", value, sys::mrb_args_none());
            spec.mrb_value_is_rust_backed(true);
        }
        let api = interp.borrow();
        let spec = api.class_spec::<Self>();
        spec.define(&interp).expect("class install");
    }
}

#[cfg(test)]
mod tests {
    use mruby::interpreter::MrbApi;

    use super::*;

    #[test]
    fn define_rust_backed_ruby_class() {
        env_logger::Builder::from_env("MRUBY_LOG").init();

        let mut interp = Interpreter::create().expect("mrb init");
        interp.def_file_for_type::<_, Container>("container");

        let code = "require 'container'; Container.new(15).value";
        let result = interp.eval(code).expect("no exceptions");
        let cint = unsafe { i64::try_from_mrb(&interp, result).expect("convert") };
        assert_eq!(cint, 15);

        drop(interp);
    }
}
