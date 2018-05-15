// pub fn merge<'a, T>(a: &'a [T], b: &'a [T]) -> &'a [T] {
//     match (a.len(), b.len()) {
//         (_, 0) => a,
//         (0, _) => b,
//         (a_len, b_len) => unsafe {
//             let a_last: *const T = a.get_unchecked(a_len);
//             let b_first: *const T = b.get_unchecked(0);;
//             if a_last != b_first {
//                 panic!("the two slices are not adjacent: {:?}, {:?}", a_last, b_first);
//             }

//             let a_first: *const T = a.get_unchecked(0);
//             ::std::slice::from_raw_parts(a_first, a_len + b_len)
//         }
//     }
// }
