use crate::component::{Component, InitNecessary, RenderNecessary, UpdateNecessary};
use std::any::{Any,TypeId};
use crate::object::Object;
use gen_code::{gen_impl_comp_common,gen_impl_as_any,AsAny};
use cgmath::{Vector3, Matrix4};
use std::rc::{Rc, Weak};
use std::ops::{Index, IndexMut};
use crate::AsAny;

#[derive(AsAny)]
pub struct Transform{
    object:*const Object,
    children:Vec<Rc<Transform>>,
    parent:Option<Weak<Transform>>,
    pub position:Vector3<f32>,
    pub rotation:Vector3<f32>,
    pub scale:Vector3<f32>,
}

impl Transform {
    pub fn new() -> Transform{
        Transform{
            object: 0 as _,
            children:Vec::new(),
            parent:None,
            position:Vector3::new(0f32,0f32,0f32),
            rotation:Vector3::new(0f32,0f32,0f32),
            scale:Vector3::new(1f32,1f32,1f32)
        }
    }
    pub fn get_local_matrix(&self) -> Matrix4<f32>
    {
        cgmath::Matrix4::from_translation(self.position) *
                Matrix4::from_angle_x(cgmath::Rad(self.rotation.x)) *
                Matrix4::from_angle_y(cgmath::Rad(self.rotation.y)) *
                Matrix4::from_angle_z(cgmath::Rad(self.rotation.z)) *
                Matrix4::from_nonuniform_scale(self.scale.x,self.scale.y,self.scale.z)
    }
    pub fn get_world_matrix(&self)->Matrix4<f32>
    {
        if let Some(p) = self.get_parent(){
            p.get_world_matrix() * self.get_local_matrix()
        }else{
            self.get_local_matrix()
        }
    }
    pub fn rm_child(&mut self,c:Rc<Transform>) -> Option<Rc<Transform>>
    {
        let mut idx = -1isize;
        for i in 0..self.children.len()
        {
            if (self.children[i].as_ref() as *const Transform) == c.as_ref() as *const Transform {
                idx = i as _;
                break;
            }
        };
        if idx >= 0 && (idx as usize) < self.children.len(){
            let mut ch = self.children.remove(idx as _);
            Rc::get_mut(&mut ch).unwrap().parent = None;
            Some(ch)
        }else{
            None
        }
    }
    pub fn remove(&mut self,i:usize) -> Option<Rc<Transform>>
    {
        if i < self.children.len() {
            let mut ch = self.children.remove(i);
            Rc::get_mut(&mut ch).unwrap().parent = None;
            Some(ch)
        }else{
            None
        }
    }
    pub fn add_child(mut self:Rc<Transform>,mut c:Rc<Transform>)->bool
    {
        let chp = c.as_ref() as *const Transform;
        if chp == self.as_ref() as *const Transform {return false;}
        if let Some(parent) = &self.parent{
            if parent.as_ptr() == chp { return false; }
        }
        if let Some(mut parent) = c.get_parent()
        {
            Rc::get_mut(&mut parent).unwrap().rm_child(c.clone());
        }
        Rc::get_mut(&mut c).unwrap().parent = Some(Rc::downgrade(&self));
        Rc::get_mut(&mut self).unwrap().children.push(c);
        true
    }
    pub fn get_child(&self,i:usize) -> Option<Rc<Transform>>
    {
        if i < self.children.len() {
            Some(self.children[i].clone())
        }else{
            None
        }
    }

    fn set_parent(&mut self,p:Weak<Transform>)
    {
        self.parent = Some(p);
    }
    pub fn get_parent(&self) -> Option<Rc<Transform>>
    {
        if let Some(parent) = &self.parent
        {
            parent.upgrade()
        }else{
            None
        }
    }
    pub fn has_parent(&self)->bool{
        !self.parent.is_some()
    }
}

impl Index<usize> for Transform {
    type Output = Rc<Transform>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.children[index]
    }
}

impl IndexMut<usize> for Transform {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.children[index]
    }
}

impl Component for Transform
{
    gen_impl_comp_common!{object}

    fn on_add(&mut self) {

    }

    fn on_remove(&mut self) {

    }

    fn init(&mut self, _nec: InitNecessary<'_>) {
        todo!()
    }

    fn render(&mut self, _nec: RenderNecessary<'_>) {
        todo!()
    }

    fn start(&mut self) {
        todo!()
    }

    fn update(&mut self, _nec: UpdateNecessary<'_>) {
        todo!()
    }


    fn destroy(&mut self) {
        todo!()
    }
}
