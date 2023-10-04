use ::libc;
extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;
    fn strdup(_: *const libc::c_char) -> *mut libc::c_char;
}
pub type size_t = libc::c_ulong;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct item {
    pub key: *mut libc::c_char,
    pub value: *mut libc::c_void,
}
impl Default for item {
    fn default() -> Self {
        Self {
            key: std::ptr::null_mut(),
            value: std::ptr::null_mut(),
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct ErasedByRefactorer0;
#[repr(C)]
pub struct table {
    pub entries: *mut item,
    pub capacity: size_t,
    pub length: size_t,
}
impl Default for table {
    fn default() -> Self {
        Self {
            entries: std::ptr::null_mut(),
            capacity: Default::default(),
            length: Default::default(),
        }
    }
}
impl table {
    pub fn take(&mut self) -> Self {
        core::mem::take(self)
    }
}

#[no_mangle]
pub unsafe extern "C" fn table_init() -> Option<Box<table>> {
    let mut ret: Option<Box<table>> = None;
    ret = Some(Box::new(<crate::array::table as Default>::default()));
    if ret.as_deref().is_none() {
        ();
        return None;
    }
    (*ret.as_deref_mut().unwrap()).entries = malloc(
        (32 as libc::c_int as libc::c_ulong)
            .wrapping_mul(::core::mem::size_of::<item>() as libc::c_ulong),
    ) as *mut item;
    if (*ret.as_deref().unwrap()).entries.is_null() {
        ();
        ();
        return None;
    }
    (*ret.as_deref_mut().unwrap()).capacity = 32 as libc::c_int as size_t;
    (*ret.as_deref_mut().unwrap()).length = 0 as libc::c_int as size_t;
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn table_fini(mut table: Option<Box<table>>) {
    if table.as_deref().is_none() {
        ();
        return;
    }
    let mut i: size_t = 0 as libc::c_int as size_t;
    while i < (*table.as_deref().unwrap()).length {
        free((*(*table.as_deref().unwrap()).entries.offset(i as isize)).key as *mut libc::c_void);
        (*(*table.as_deref().unwrap()).entries.offset(i as isize)).key = 0 as *mut libc::c_char;
        i = i.wrapping_add(1);
        i;
    }
    free((*table.as_deref().unwrap()).entries as *mut libc::c_void);
    (*table.as_deref_mut().unwrap()).entries = 0 as *mut item;
    ();
}
#[no_mangle]
pub unsafe extern "C" fn table_set(
    mut table: Option<&mut table>,
    mut key: *mut libc::c_char,
    mut value: *mut libc::c_void,
) -> *mut libc::c_char {
    if table.as_deref().is_none() {
        ();
        return 0 as *mut libc::c_char;
    }
    let mut i: size_t = 0 as libc::c_int as size_t;
    while i < (*table.as_deref().unwrap()).length {
        if 0 as libc::c_int
            == strcmp(
                key,
                (*(*table.as_deref().unwrap()).entries.offset(i as isize)).key,
            )
        {
            return (*(*table.as_deref().unwrap()).entries.offset(i as isize)).key;
        }
        i = i.wrapping_add(1);
        i;
    }
    if (*table.as_deref().unwrap()).capacity <= (*table.as_deref().unwrap()).length {
        return 0 as *mut libc::c_char;
    }
    (*table.as_deref_mut().unwrap()).length = (*table.as_deref().unwrap()).length.wrapping_add(1);
    let mut idx: size_t = (*table.as_deref().unwrap()).length;
    (*(*table.as_deref().unwrap()).entries.offset(idx as isize)).key = strdup(key);
    (*(*table.as_deref().unwrap()).entries.offset(idx as isize)).value = value;
    return (*(*table.as_deref().unwrap()).entries.offset(idx as isize)).key;
}
#[no_mangle]
pub unsafe extern "C" fn table_get(
    mut table: *mut table,
    mut key: *mut libc::c_char,
) -> *mut libc::c_void {
    if table.is_null() {
        ();
        return 0 as *mut libc::c_void;
    }
    let mut i: size_t = 0 as libc::c_int as size_t;
    while i < (*table).length {
        if 0 as libc::c_int == strcmp(key, (*(*table).entries.offset(i as isize)).key) {
            return (*(*table).entries.offset(i as isize)).value;
        }
        i = i.wrapping_add(1);
        i;
    }
    return 0 as *mut libc::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn buggy_called(
    mut table: Option<&mut table>,
    mut item: *mut item,
) -> *mut table {
    (*table.as_deref_mut().unwrap()).capacity = 4 as libc::c_int as size_t;
    return table
        .as_deref_mut()
        .map(|r| r as *mut _)
        .unwrap_or(std::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "C" fn buggy_call(mut table: Option<&mut table>) {
    let mut x: *mut table = table
        .as_deref_mut()
        .map(|r| r as *mut _)
        .unwrap_or(std::ptr::null_mut());
    buggy_called(table.as_deref_mut(), (*table.as_deref().unwrap()).entries);
}
