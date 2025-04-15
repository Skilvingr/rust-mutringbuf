extern crate alloc;

use crate::{common_def, get_buf};
use mutringbuf::iterators::ProdIter;
use mutringbuf::{MRBIterator, MutRB};

common_def!();
const MULTIPLE: usize = 42;

fn fill_buf(prod: &mut ProdIter<impl MutRB<Item = usize>>) {
    let slice = (0..BUFFER_SIZE - 1).collect::<Vec<usize>>();
    prod.push_slice(&slice);
}

#[test]
fn test_work_single() {
    let (mut prod, mut work, mut cons) = get_buf!(Concurrent).split_mut();

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    fill_buf(&mut prod);

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
fn test_work_mul() {
    let (mut prod, mut work, mut cons) = get_buf!(Concurrent).split_mut();

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    fill_buf(&mut prod);

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE - 1);
    assert_eq!(cons.available(), 0);

    if let Some(res) = work.get_workable_slice_multiple_of(MULTIPLE) {
        res.iter_mut().for_each(|v| *v += 1);
        unsafe { work.advance(res.len()) };
    }

    let rem = BUFFER_SIZE % MULTIPLE;
    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), rem - 1);
    assert_eq!(cons.available(), BUFFER_SIZE - rem);

    for i in 0..BUFFER_SIZE - rem {
        assert_eq!(*cons.peek_ref().unwrap(), i + 1);
        unsafe { cons.advance(1); }
    }

    assert_eq!(prod.available(), BUFFER_SIZE - rem);
    assert_eq!(work.available(), rem - 1);
    assert_eq!(cons.available(), 0);

    if let Some(res) = work.get_workable_slice_avail() {
        res.iter_mut().for_each(|v| *v += 1);
        unsafe { work.advance(res.len()) };
    }

    assert_eq!(prod.available(), BUFFER_SIZE - rem);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), rem - 1);

    for i in BUFFER_SIZE - rem..BUFFER_SIZE - 1 {
        assert_eq!(cons.pop().unwrap(), i + 1);
    }

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);
}
#[test]
fn test_work_exact() {
    let (mut prod, mut work, mut cons) = get_buf!(Concurrent).split_mut();

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    fill_buf(&mut prod);

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE - 1);
    assert_eq!(cons.available(), 0);

    let step = 10;
    let max = 30;
    for _ in 0..max {
        if let Some(res) = work.get_workable_slice_exact(step) {
            res.iter_mut().for_each(|v| *v += 1);
            unsafe { work.advance(res.len()) };
        }
    }

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE - 1 - max * step);
    assert_eq!(cons.available(), max * step);

    for i in 0..cons.available() {
        assert_eq!(cons.pop().unwrap(), i + 1);
    }

    assert_eq!(prod.available(), max * step);
    assert_eq!(work.available(), BUFFER_SIZE - 1 - max * step);
    assert_eq!(cons.available(), 0);
}