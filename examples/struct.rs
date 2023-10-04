
struct s {
    a: Option<Box<i32>>,
    b: Option<&mut i32>,
    c: Option<&i32>,
    d: *const i32,
    e: *mut /* owning */ i32,
    f: *mut i32,
}