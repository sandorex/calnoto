use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct TestClass;

#[godot_api]
impl TestClass {
    #[func]
    fn clone(&mut self, path: String) {
        use git2::Repository;

        let url = "https://github.com/sandorex/icalendar.txt";
        let repo = match Repository::clone(url, &path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };
    }
}

#[godot_api]
impl INode for TestClass {
    fn init(_base: Base<Node>) -> Self {
        godot_print!("Got initialized");

        Self
    }
}


// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
