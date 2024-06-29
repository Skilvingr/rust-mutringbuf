extern crate alloc;

use mutringbuf::{ConcurrentHeapRB, HeapSplit, MRBIterator, ProdIter};



const BUFFER_SIZE: usize = 100;

fn fill_buf(prod: &mut ProdIter<ConcurrentHeapRB<usize>>) {
    let slice = (0..BUFFER_SIZE).collect::<Vec<usize>>();
    prod.push_slice(&slice);
}

#[test]
fn test_work_single() {
    let (mut prod, mut work, mut cons) = ConcurrentHeapRB::default(BUFFER_SIZE + 1).split_mut();

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    fill_buf(&mut prod);

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
fn test_work_mul() {
    let (mut prod, mut work, mut cons) = ConcurrentHeapRB::default(BUFFER_SIZE + 1).split_mut();

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    fill_buf(&mut prod);

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE);
    assert_eq!(cons.available(), 0);

    if let Some((h, t)) = work.get_workable_slice_multiple_of(42) {

        let len = h.len() + t.len();

        h.iter_mut().for_each(|v| *v += 1);
        t.iter_mut().for_each(|v| *v += 1);

        unsafe { work.advance(len) };
    }

    // 42 * 2 = 84 => rem = 100 - 84 = 16
    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 16);
    assert_eq!(cons.available(), 84);

    for i in 0..84 {
        assert_eq!(*cons.peek_ref().unwrap(), i + 1);
        unsafe { cons.advance(1); }
    }

    assert_eq!(prod.available(), 84);
    assert_eq!(work.available(), 16);
    assert_eq!(cons.available(), 0);

    if let Some((h, t)) = work.get_workable_slice_avail() {

        let len = h.len() + t.len();

        h.iter_mut().for_each(|v| *v += 1);
        t.iter_mut().for_each(|v| *v += 1);

        unsafe { work.advance(len) };
    }

    assert_eq!(prod.available(), 84);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 16);

    for i in 84..BUFFER_SIZE {
        unsafe { assert_eq!(cons.pop().unwrap(), i + 1); }
    }

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);
}
#[test]
fn test_work_exact() {
    let (mut prod, mut work, mut cons) = ConcurrentHeapRB::default(BUFFER_SIZE + 1).split_mut();

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    fill_buf(&mut prod);

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE);
    assert_eq!(cons.available(), 0);

    for _ in 0..20 {
        if let Some((h, t)) = work.get_workable_slice_exact(5) {
            let len = h.len() + t.len();

            h.iter_mut().for_each(|v| *v += 1);
            t.iter_mut().for_each(|v| *v += 1);

            unsafe { work.advance(len) };
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