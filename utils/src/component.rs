use wgpu::{Device, Queue, SwapChainDescriptor};
use std::any::{TypeId, Any};
use crate::object::Object;
use crate::AsAny;

pub struct InitNecessary<'a>{
    pub device:&'a Device,
    pub queue:&'a Queue,
    pub swap_chain_desc:&'a SwapChainDescriptor
}
pub struct RenderNecessary<'a>{
    pub swap_chain_desc:&'a SwapChainDescriptor
}
pub struct UpdateNecessary<'a>{
    pub delat:f32,
    pub device:&'a Device,
    pub queue:&'a Queue,
    pub swap_chain_desc:&'a SwapChainDescriptor
}

pub trait Component : AsAny{
    fn type_id(&self) -> TypeId { self.as_any().type_id() }

    fn on_reg(&mut self,obj:*const Object);
    fn on_unreg(&mut self);
    fn on_add(&mut self);
    fn on_remove(&mut self);

    fn object(&self) -> &Object;
    fn mut_object(&self) -> &mut Object;

    fn priority(&self) -> i32 { 0 }
    fn init(&mut self,nec:InitNecessary<'_>);
    fn render(&mut self,nec:RenderNecessary<'_>);
    fn start(&mut self);
    fn update(&mut self,nec:UpdateNecessary<'_>);
    fn destroy(&mut self);

}