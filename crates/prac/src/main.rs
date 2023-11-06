trait A<In, Out> {
    fn a(&self) {}
}

impl A<f32, ()> for fn(f32) {}

fn main() {
    //build error
    // test_f32.a();

    (test_f32 as fn(f32)).a();
}

fn test_f32(_: f32) {}
