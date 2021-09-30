extern crate proc_macro;
extern crate syn;

use syn::{
    Ident
};
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

#[proc_macro]
pub fn gen_impl_res_process_cache(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let obj_name = Ident::new(input.to_string().as_str(),Span::call_site());

    let tokens = quote::quote!{
        fn get_cache(&self, path: &String) -> Option<Rc<Self::Out>> {
            if let Some(v) = self.#obj_name.get(path)
            {
                Some(v.clone())
            }else{
                None
            }
        }

        fn add_cache(&mut self, path: String, data: Rc<Self::Out>) {
            self.#obj_name.insert(path,data);
        }

        fn clear_cache(&mut self) {
            self.#obj_name.clear();
        }

        fn rm_cache(&mut self, path: &String) -> Option<Rc<Self::Out>> {
            self.#obj_name.remove(path)
        }
        fn as_any(self:Box<Self>) -> Box<dyn Any>
        {
            self
        }
    };

    tokens.into()
}