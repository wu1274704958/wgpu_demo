extern crate proc_macro;
extern crate syn;

use syn::{
    Lit,Ident
};
use std::any::Any;
use proc_macro2::Span;

#[proc_macro]
pub fn gen_impl_comp_common(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let obj_name = Ident::new(input.to_string().as_str(),Span::call_site());


    let tokens = quote::quote!{
        fn as_any(&self) -> &dyn Any {self}

        fn as_mut_any(&mut self) -> &mut dyn Any {self}

        fn on_reg(&mut self, obj: *const Object) {
            self.#obj_name = obj;
            self.on_add();
        }

        fn on_unreg(&mut self) {
            self.on_unreg();
            self.#obj_name = 0 as _;
        }

        fn object(&self) -> &Object {
            assert!(!self.#obj_name.is_null());
            unsafe { std::mem::transmute::<*const Object,&Object>(self.#obj_name) }
        }

        fn mut_object(&self) -> &mut Object {
            assert!(!self.#obj_name.is_null());
            unsafe { std::mem::transmute::<*const Object,&mut Object>(self.#obj_name) }
        }
    };

    tokens.into()
}