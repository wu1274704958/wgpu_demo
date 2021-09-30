use std::path::Path;
use std::rc::Rc;
use std::collections::HashMap;
use gen_code::gen_impl_res_process_cache;
use std::any::{TypeId, Any};
use std::time::SystemTime;
use std::io::Read;

pub trait ResProcesser
{
    type In;
    type Out;
    fn process(&self,d:Rc<Self::In>) -> Option<Rc<Self::Out>>;
    fn process_cache(&mut self,path:&String,d:Rc<Self::In>,cache_overdue:bool) -> Option<Rc<Self::Out>>
    {
        if cache_overdue { self.rm_cache(path);}
        if let Some(data) = self.get_cache(path){
            Some(data)
        }else{
            if let Some(data) = self.process(d){
                self.add_cache(path.clone(),data.clone());
                Some(data)
            }else { None }
        }
    }

    fn get_cache(&self,path:&String) -> Option<Rc<Self::Out>>;
    fn add_cache(&mut self,path:String,data:Rc<Self::Out>);
    fn clear_cache(&mut self);
    fn rm_cache(&mut self,path:&String) -> Option<Rc<Self::Out>>;
    fn as_any(self:Box<Self>) -> Box<dyn Any>;

}

pub struct TextRes{
    cache: HashMap<String,Rc<String>>,
}

impl TextRes {
    pub fn new() ->TextRes
    {
        TextRes{
            cache:Default::default()
        }
    }
}

impl ResProcesser for TextRes {
    type In = Vec<u8>;
    type Out = String;

    gen_impl_res_process_cache!{cache}

    fn process(&self, d: Rc<Self::In>) -> Option<Rc<String>> {
        if let Ok(v) = String::from_utf8(d.as_ref().clone())
        {
            Some(Rc::new(v))
        }else{
            None
        }
    }
}


pub struct ResourceMgr{
    root: String,
    cache: HashMap<String,(Rc<Vec<u8>>,SystemTime)>,
    process: HashMap<TypeId,Box<dyn Any>>,
}

impl ResourceMgr {
    pub fn new(root:String) -> ResourceMgr{
        ResourceMgr{
            root,
            cache: Default::default(),
            process: Default::default()
        }
    }
    pub fn get_cache(&self,path:&String) -> Option<&(Rc<Vec<u8>>,SystemTime)>
    {
        self.cache.get(path)
    }
    pub fn add_cache(&mut self,path:String,data:Rc<Vec<u8>>,time:SystemTime)
    {
        self.cache.insert(path,(data,time));
    }
    pub fn clear_cache(&mut self)
    {
        self.cache.clear();
    }
    pub fn rm_cache(&mut self,path:&String) -> Option<(Rc<Vec<u8>>,SystemTime)>
    {
        self.cache.remove(path)
    }
    pub fn load_file(&mut self,p:&str) -> Option<(Rc<Vec<u8>>,bool,String)>
    {
        let root = Path::new(self.root.as_str());
        let path = root.join(p);
        let path_str = if let Some(str) = path.to_str(){
            str.to_string()
        }else { return None; };

        if let Ok(mut file) = std::fs::OpenOptions::new().read(true).open(path)
        {
            let modify_time = if let Ok(meta) = file.metadata()
            {
                if let Ok(t) = meta.modified(){ t } else { SystemTime::UNIX_EPOCH }
            }else {
                SystemTime::UNIX_EPOCH
            };

            let has_cache = if let Some((cache,time)) = self.get_cache(&path_str)
            {
                if *time >= modify_time {
                    return Some((cache.clone(),false,path_str));
                }else {
                    true
                }
            }else{
                false
            };
            let mut data = Vec::new();
            if let Ok(len) = file.read_to_end(&mut data)
            {
                if len > 0{
                    let d = Rc::new(data);
                    self.add_cache(path_str.clone(),d.clone(),modify_time);
                    Some((d,has_cache,path_str))
                }else{None}
            }else{
                None
            }
        }else{
            None
        }
    }

    pub fn add_process<T:ResProcesser<In = I,Out = O>,I,O>(&mut self,p:Box<T>)
        where O : 'static, I : 'static,T :'static,
              T : ResProcesser<In = I,Out = O>
    {
        self.process.insert(TypeId::of::<O>(),p.as_any());
    }

    pub fn loading<T:ResProcesser<In = I,Out = O>,I,O>(&mut self,i:Rc<I>,path:&String,cache_overdue:bool) -> Option<Rc<O>>
    where O : 'static, I : 'static,T :'static,
    T : ResProcesser<In = I,Out = O>
    {
        let proc = if let Some(v) = self.process.get_mut(&TypeId::of::<O>())
        {
            if let Some(p) = v.downcast_mut::<T>(){ p }else { return None; }
        }else{
            return None;
        };
        proc.process_cache(path,i,cache_overdue)
    }

}

#[macro_export]
macro_rules! load_chain
{
    ($mgr:ident,$path:expr) => {
        if let Some((res,b,s)) = $mgr.load_file($path)
        {
            Some(res)
        }else{
            None
        }
    };
    ($mgr:ident,$path:expr,$($T:tt)*) => {
        if let Some((res,b,s)) = $mgr.load_file($path)
        {
            Some(res)
            let v = $mgr.loading::<$T,_,_>(res,&s,b).unwrap();
        }else{
            None
        }
    };
}

mod test_load_file{
    use crate::resource_manager::{ResourceMgr, TextRes};
    use std::path::Path;
    use std::process::Command;
    use std::io::Write;
    use std::fs::OpenOptions;

    #[test]
    fn test()
    {
        let mut mgr = ResourceMgr::new("".to_string());
        mgr.add_process(Box::new(TextRes::new()));
        let (res,b,s) = mgr.load_file("test_load.txt").unwrap();
        let v = mgr.loading::<TextRes,_,_>(res,&s,b).unwrap();
        dbg!(v);

        dbg!(load_chain!(mgr,"test_load.txt"));
    }
}

