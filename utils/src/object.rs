use crate::component::Component;
use std::collections::{ HashMap};
use std::any::{Any, TypeId};
use std::marker::PhantomPinned;
use std::pin::Pin;

pub struct Object
{
    components:Vec<Box<dyn Component>>,
    comp_type_set: HashMap<TypeId,usize>,
    _pined : PhantomPinned,
}

impl Object
{
    pub fn new()-> Pin<Box<Object>>{
        let obj = Object{
            components: Vec::new(),
            comp_type_set: HashMap::<TypeId,usize>::new(),
            _pined: PhantomPinned
        };
        Box::pin(obj)
    }
    pub fn add_comp<T>(&mut self,c:Box<T>) -> bool
        where T:Component + 'static
    {
        if !self.has_comp::<T>()
        {
            return self.add_with_sort(c,Self::get_comp_hash::<T>());
        }
        return false;
    }

    pub fn add_comp_dyn(&mut self,c:Box<dyn Component>) -> bool
    {
        let ty = c.type_id();
        if !self.comp_type_set.contains_key(&ty)
        {
            return self.add_with_sort(c,ty);
        }
        return false;
    }

    pub fn has_comp<T>(&self)->bool
        where T:Component + 'static
    {
        self.comp_type_set.contains_key(&Self::get_comp_hash::<T>())
    }

    pub fn get_comp_hash<T>()->TypeId
        where T:Component + 'static
    {
        TypeId::of::<T>()
    }

    pub fn rm_comp<T>(&mut self) -> Option<Box<dyn Component>>
        where T:Component + 'static
    {
        if let Some(idx) = self.comp_type_set.remove(&Self::get_comp_hash::<T>())
        {
            if idx < self.components.len(){
                let mut c = self.components.remove(idx);
                c.on_unreg();
                return Some(c);
            }
        }
        None
    }

    pub fn get_comp<T>(&self) -> Option<&Box<dyn Component>>
        where T:Component + 'static
    {
        if let Some(idx) = self.comp_type_set.get(&Self::get_comp_hash::<T>())
        {
            return self.components.get(*idx);
        }
        None
    }

    pub fn get_comp_mut<T>(&mut self) -> Option<&mut Box<dyn Component>>
        where T:Component + 'static
    {
        if let Some(idx) = self.comp_type_set.get(&Self::get_comp_hash::<T>())
        {
            return self.components.get_mut(*idx);
        }
        None
    }

    fn self_ptr(&self) -> *const Object{
        self as *const Object
    }

    fn add_with_sort(&mut self,mut c:Box<dyn Component>,ty:TypeId) -> bool
    {
        let mut idx = -1isize;
        for i in 0..self.components.len(){
            if c.priority() < self.components[i].priority()
            {
                idx = i as _;break;
            }
        }
        if idx >= 0{
            c.on_reg(self.self_ptr());
            self.components.insert(idx as usize,c);
            self.comp_type_set.insert(ty,idx as usize);
            for i in (idx + 1) as usize..self.components.len(){
                if let Some(index) = self.comp_type_set.get_mut(&self.components[i].type_id())
                {
                    *index += 1;
                }
            }
        }else{
            c.on_reg(self.self_ptr());
            self.components.push(c);
            self.comp_type_set.insert(ty,self.components.len() - 1);
        }
        return true;
    }
    pub fn pin_get(self:Pin<&mut Self>) -> &mut Object
    {
        unsafe {self.get_unchecked_mut()}
    }
}


mod test_object{
    use crate::object::Object;
    use crate::components::Transform;
    use crate::component::Component;

    #[test]
    fn test()
    {
        let mut obj = Object::new();
        obj.as_mut().pin_get().add_comp(Box::new(Transform::new()));
        let trans = obj.get_comp::<Transform>().unwrap();
        let obj_ptr1 = trans.object() as *const Object;

        let obj_moved = obj;
        let trans = obj_moved.get_comp::<Transform>().unwrap();
        let obj_ptr2 = trans.object();

        let trans2 = obj_ptr2.get_comp::<Transform>().unwrap();

        assert_eq!(obj_ptr1,obj_ptr2 as *const Object);
        assert_eq!(trans2.as_ref() as *const dyn Component,trans.as_ref() as *const dyn Component);
    }

    #[test]
    fn test2()
    {
        let mut obj = Object{
            components: Vec::new(),
            comp_type_set: Default::default(),
            _pined: Default::default()
        };
        obj.add_comp(Box::new(Transform::new()));
        let trans = obj.get_comp::<Transform>().unwrap();
        let obj_ptr1 = trans.object() as *const Object;

        let obj_moved = obj;
        let _obj_moved_ptr = &obj_moved as *const Object;
        let trans = obj_moved.get_comp::<Transform>().unwrap();
        let obj_ptr2 = trans.object();
        let arr = [0i32;50];
        let trans2 = t2_in(arr,obj_ptr2);

        assert_eq!(obj_ptr1,obj_ptr2 as *const Object);
        assert_eq!(trans2.as_ref() as *const dyn Component,trans.as_ref() as *const dyn Component);
    }

    fn t2_in(_arr:[i32;50],obj:&Object) -> &Box<dyn Component>
    {
        obj.get_comp::<Transform>().unwrap()
    }
}
