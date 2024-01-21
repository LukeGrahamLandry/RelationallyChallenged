pub mod ir;
pub mod lex;
pub mod parse;
pub mod print;
pub mod vm;

// #[cfg(target_arch = "wasm32")]
pub mod web {
    //! C ABI callable from js when targeting wasm.

    use crate::lex::Lexer;
    use crate::parse::{ParseErr, Parser};
    use crate::vm::Vm;
    use std::alloc::{alloc, Layout};
    use std::ffi::{c_char, CStr, CString};
    use std::ptr::slice_from_raw_parts;

    /// len does NOT include null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn run_query(input_query: *const u8, len: usize) -> *const c_char {
        let src = &*slice_from_raw_parts(input_query, len);
        let res = 'res: {
            let src = match String::from_utf8(src.to_vec()) {
                Ok(src) => src,
                Err(e) => break 'res e.to_string(),
            };

            let mut output = String::new();

            let mut vm = Vm::default();
            let mut parse = Parser::new(Lexer::new(&src));
            let mut next = parse.next();
            const SEP: &str = "~~~|||@@@|||~~~";
            // TODO: make parser an iterator so this is less ugly
            while next.is_ok() {
                let ast = next.unwrap();
                output += &format!("ok{SEP}{ast:#?}{SEP}");
                match vm.compile_impure(ast) {
                    Ok((q, _)) => {
                        let res = vm.eval(&q);
                        let printed = &vm.print_rows_iter(res);
                        output += &format!("ok{SEP}{q:#?}{SEP}{printed}{SEP}");
                    }
                    Err(e) => {
                        output += &format!("err{SEP}{e:?}{SEP}");
                    }
                }
                next = parse.next();
            }
            match next {
                Err(ParseErr::EOF) => {}
                Err(e) => {
                    output += &format!("err{SEP}{e:?}\n");
                    if let Some(span) = e.get_span() {
                        let slice = if span.end < src.len() {
                            &src[span.start..span.end]
                        } else {
                            "ICE: token slice OOB"
                        };
                        output += &format!(
                            "error at source bytes {0} to {1}:\n{2}\n--- end of error token ---",
                            span.start, span.end, slice
                        );
                    }
                }
                Ok(_) => unreachable!(), // TODO: move match into loop so this is obvious
            }
            break 'res output;
        };

        CString::new(res).unwrap().into_raw()
    }

    /// len DOES include null terminator
    #[no_mangle]
    pub unsafe extern "C" fn alloc_str(len: usize) -> *mut u8 {
        alloc(Layout::array::<u8>(len).unwrap())
    }

    #[no_mangle]
    pub unsafe extern "C" fn drop_c_str(ptr: *mut c_char) {
        let _ = CString::from_raw(ptr);
    }

    /// result does NOT include null terminator.
    #[no_mangle]
    pub unsafe extern "C" fn c_str_len(ptr: *mut c_char) -> usize {
        CStr::from_ptr(ptr).to_bytes().len()
    }
}
