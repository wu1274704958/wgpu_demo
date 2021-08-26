
fn main() {
    let data = include_bytes!("red.jpg");
    let mut f = false;
    let res:Vec<u16> = data.chunks(2)
        .filter_map(|it|{
            if f || it.len() < 2 {return None;}
            let buf = [it[0],it[1]];
            let val = u16::from_be_bytes(buf);
            //if val == 0xffdb {  f = true; }
            return Some(val);
    }).collect();
    res.chunks(20).for_each(|it|
    {
        it.iter().for_each(|i|{print!("{:x}, ",*i)});
        println!();
    });
}