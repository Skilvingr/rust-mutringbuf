#[cfg(feature = "alloc")]
#[test]
fn test_uninit() {
    use crate::common_def;
    use mutringbuf::{ConcurrentHeapRB, HeapSplit, MRBIterator};
    use std::rc::Rc;

    common_def!(buf);

    // Indices from 0 to RB_SIZE+1 are uninitialised
    let buf = unsafe { ConcurrentHeapRB::new_zeroed(BUFFER_SIZE) };
    let (mut prod, mut cons) = buf.split();

    let slice = (0..BUFFER_SIZE - 1)
        .map(Rc::new)
        .collect::<Vec<Rc<usize>>>();

    for x in &slice {
        prod.push_init(x.clone()).unwrap();
    } // RB_SIZE indices out of RB_SIZE+1 are initialised.

    unsafe {
        cons.advance(BUFFER_SIZE - 1);
    }

    for x in &slice {
        prod.push_init(x.clone()).unwrap();
    } // All indices are now initialised. It would be now safe to use normal methods from prod instead of `*_init` ones.

    for _ in &slice {
        unsafe {
            cons.pop_move().unwrap();
        }
    } // 1 index out of RB_SIZE+1 is initialised. `*_init` methods must be used.

    for x in &slice {
        prod.push_init(x.clone()).unwrap();
    } // RB_SIZE indices out of RB_SIZE+1 are initialised.

    unsafe {
        cons.advance(BUFFER_SIZE - 1);
    }

    prod.push_slice_clone_init(&slice).unwrap(); // All indices are now initialised. It would be now safe to use normal methods from prod instead of `*_init` ones.

    unsafe {
        cons.advance(BUFFER_SIZE - 1);
    }

    prod.push_slice_clone(&slice).unwrap();

    for _ in &slice {
        unsafe {
            cons.pop_move().unwrap();
        }
    }

    drop(prod);
    drop(cons);
}
