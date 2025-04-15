#[cfg(feature = "alloc")]
fn main() {

    use std::rc::Rc;

    use mutringbuf::{ConcurrentHeapRB, HeapSplit, MRBIterator};

    const RB_SIZE: usize = 4095;
    
    // Indices from 0 to RB_SIZE+1 are uninitialised
    let buf = unsafe { ConcurrentHeapRB::new_zeroed(RB_SIZE + 1) };
    let (mut prod, mut cons) = buf.split();

    let slice = (0..RB_SIZE).map(Rc::new).collect::<Vec<Rc<usize>>>();

    for x in &slice {
        prod.push_init(x.clone()).unwrap();
    } // RB_SIZE indices out of RB_SIZE+1 are initialised.

    unsafe { cons.advance(RB_SIZE); }

    for x in &slice {
        prod.push_init(x.clone()).unwrap();
    } // All indices are now initialised. It would be now safe to use normal methods from prod instead of `*_init` ones.

    for _ in &slice {
        unsafe { cons.pop_move().unwrap(); }
    } // 1 index out of RB_SIZE+1 is initialised. `*_init` methods must be used.

    for x in &slice {
        prod.push_init(x.clone()).unwrap();
    } // RB_SIZE indices out of RB_SIZE+1 are initialised.

    unsafe { cons.advance(RB_SIZE); }

    prod.push_slice_clone_init(&slice).unwrap(); // All indices are now initialised. It would be now safe to use normal methods from prod instead of `*_init` ones.

    unsafe { cons.advance(RB_SIZE); }

    prod.push_slice_clone(&slice).unwrap();

    for _ in &slice {
        unsafe { cons.pop_move().unwrap(); }
    }

    drop(prod);
    drop(cons);
}

#[cfg(not(feature = "alloc"))]
fn main() {}
