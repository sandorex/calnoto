// mod cxx {
//     use libc::c_int;
//
//     unsafe extern "C" {
//         pub safe fn qt_start() -> c_int;
//     }
// }

// fn main() {
//     println!("Hello, world!");
//     let code = cxx::qt_start();
//
//     eprintln!("code: {code}");
//     if code != 0 {
//         std::process::exit(1);
//     }
// }

#[unsafe(no_mangle)]
pub unsafe extern "C" fn get_number() -> i32 {
    return 45;
}
