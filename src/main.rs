use std::any::Any;

async fn run() {
    let adapter = wgpu::Instance::new(wgpu::BackendBit::PRIMARY)
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();
    println!("{:?}", adapter.get_info())
}

trait A {
    fn a(&self);
    fn as_any(&self)-> &dyn Any;
    fn as_mut_any(&mut self)-> &mut dyn Any;
}

trait B {
    fn b(&self);
}

struct Obj;

impl Obj{
    fn say(&self)
    {
        println!("Say ....");
    }
}

impl A for Obj {
    fn a(&self) {
        println!("aaa");
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl B for Obj {
    fn b(&self) {
        println!("bbb");
    }
}

fn main() {
    let mut o = Obj{};
    o.a();
    o.b();
    let mut a = as_a(Box::new(o));
    {
        let any = a.as_any();
        let o = any.downcast_ref::<Obj>().unwrap();
        o.say();
    }
    {
        let any = a.as_mut_any();
        let o = any.downcast_mut::<Obj>().unwrap();
        o.say();
    }

    pollster::block_on(run());
}

fn as_a(a:Box<dyn A>) -> Box<dyn A>
{
    a
}
