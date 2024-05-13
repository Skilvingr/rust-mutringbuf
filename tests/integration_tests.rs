extern crate alloc;

use mutringbuf::{ConcurrentHeapRB, MRBIterator, LocalHeapRB};

const BUFFER_SIZE: usize = 300;

#[test]
fn test_push_work_pop_single() {
    let (mut prod, mut work, mut cons) = ConcurrentHeapRB::default(BUFFER_SIZE + 1).split_mut();

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    for i in 0..BUFFER_SIZE {
        let _ = prod.push(i);
    }
    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE);
    assert_eq!(cons.available(), 0);

    for _ in 0..BUFFER_SIZE {
        if let Some(data) = work.get_workable() {
            *data += 1;
            unsafe { work.advance(1) };
        }
    }
    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), BUFFER_SIZE);

    for i in 0..BUFFER_SIZE {
        unsafe { assert_eq!(cons.pop().unwrap(), i + 1); }
    }

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);
}

#[test]
fn test_push_work_pop_slice() {
    let (mut prod, mut work, mut cons) = LocalHeapRB::default(BUFFER_SIZE + 1).split_mut();

    let slice = (0..BUFFER_SIZE).collect::<Vec<usize>>();

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    prod.push_slice(&slice);

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE);
    assert_eq!(cons.available(), 0);


    if let Some((h, t)) = work.get_workable_slice_exact(BUFFER_SIZE) {
        for i in h.iter_mut().chain(t) {
            *i += 1;
        }
        unsafe { work.advance(BUFFER_SIZE) };
    }


    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), BUFFER_SIZE);


    if let Some((h, t)) = cons.peek_slice(BUFFER_SIZE) {
        for (consumed, i) in [h, t].concat().iter().zip(slice) {
            assert_eq!(*consumed, i + 1);
        }
    }
    unsafe { cons.advance(BUFFER_SIZE) };

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);
}

#[test]
fn test_reset() {
    let (mut prod, mut work, mut cons) = ConcurrentHeapRB::default(BUFFER_SIZE + 1).split_mut();

    let two_thirds_slice = (0..BUFFER_SIZE/3 * 2).collect::<Vec<usize>>();
    let slice = (0..BUFFER_SIZE).collect::<Vec<usize>>();

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    prod.push_slice(&two_thirds_slice);

    assert_eq!(prod.available(), BUFFER_SIZE/3);
    assert_eq!(work.available(), BUFFER_SIZE/3 * 2);
    assert_eq!(cons.available(), 0);

    unsafe { work.advance(BUFFER_SIZE/3) };

    assert_eq!(prod.available(), BUFFER_SIZE/3);
    assert_eq!(work.available(), BUFFER_SIZE/3);
    assert_eq!(cons.available(), BUFFER_SIZE/3);

    work.reset_index();

    assert_eq!(prod.available(), BUFFER_SIZE/3);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), BUFFER_SIZE/3 * 2);

    cons.reset_index();

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    prod.push_slice(&slice);

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE);
    assert_eq!(cons.available(), 0);

    work.reset_index();

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), BUFFER_SIZE);

}