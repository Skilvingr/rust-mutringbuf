extern crate alloc;

use crate::{common_def, get_buf};
use mutringbuf::MRBIterator;

common_def!();

#[test]
fn test_push_work_pop_single() {
    let (mut prod, mut work, mut cons) = get_buf!(Concurrent).split_mut();

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    for i in 0..BUFFER_SIZE - 1 {
        let _ = prod.push(i);
    }
    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE - 1);
    assert_eq!(cons.available(), 0);

    for _ in 0..BUFFER_SIZE - 1 {
        if let Some(data) = work.get_workable() {
            *data += 1;
            unsafe { work.advance(1) };
        }
    }
    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), BUFFER_SIZE - 1);

    for i in 0..BUFFER_SIZE - 1 {
        assert_eq!(cons.pop().unwrap(), i + 1);
    }

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);
}

#[test]
fn test_push_work_pop_slice() {
    let (mut prod, mut work, mut cons) = get_buf!(Concurrent).split_mut();

    let slice = (0..BUFFER_SIZE - 1).collect::<Vec<usize>>();

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    prod.push_slice(&slice);

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE - 1);
    assert_eq!(cons.available(), 0);

    if let Some(res) = work.get_workable_slice_exact(BUFFER_SIZE - 1) {
        for i in res {
            *i += 1;
        }
        unsafe { work.advance(BUFFER_SIZE - 1) };
    }

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), BUFFER_SIZE - 1);

    if let Some(res) = cons.peek_slice(BUFFER_SIZE - 1) {
        for (consumed, i) in res.iter().zip(slice) {
            assert_eq!(*consumed, i + 1);
        }
    }
    unsafe { cons.advance(BUFFER_SIZE - 1) };

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);
}

#[test]
fn test_reset() {
    let (mut prod, mut work, mut cons) = get_buf!(Concurrent).split_mut();

    let two_thirds_slice = (0..BUFFER_SIZE / 3 * 2).collect::<Vec<usize>>();
    let slice = (0..BUFFER_SIZE - 1).collect::<Vec<usize>>();

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    prod.push_slice(&two_thirds_slice);

    assert_eq!(prod.available(), BUFFER_SIZE / 3);
    assert_eq!(work.available(), BUFFER_SIZE / 3 * 2);
    assert_eq!(cons.available(), 0);

    unsafe { work.advance(BUFFER_SIZE / 3) };

    assert_eq!(prod.available(), BUFFER_SIZE / 3);
    assert_eq!(work.available(), BUFFER_SIZE / 3);
    assert_eq!(cons.available(), BUFFER_SIZE / 3);

    work.reset_index();

    assert_eq!(prod.available(), BUFFER_SIZE / 3);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), BUFFER_SIZE / 3 * 2);

    cons.reset_index();

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    prod.push_slice(&slice);

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE - 1);
    assert_eq!(cons.available(), 0);

    work.reset_index();

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), BUFFER_SIZE - 1);
}
