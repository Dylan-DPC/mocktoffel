use assert_proc::assert_proc;
use mocktoffel::mock;

fn main() {}

// #[assert_proc]
// #[assert_duplicated = BarMock]
#[mock]
#[derive(Default)]
enum Bar {
    #[default]
    Never,
    Gonna,
    Give,
    You,
    Up,
}
