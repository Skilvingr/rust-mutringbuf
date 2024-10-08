extern crate alloc;

use mutringbuf::{ConsIter, ConcurrentHeapRB, MRBIterator, ProdIter, WorkIter, HeapSplit};
use mutringbuf::iterators::sync_iterators::detached::Detached;

const BUFFER_SIZE: usize = 100;

fn fill_buf(prod: &mut ProdIter<ConcurrentHeapRB<usize>>) {
    let slice = (0..BUFFER_SIZE).collect::<Vec<usize>>();
    prod.push_slice(&slice);
}

#[allow(clippy::type_complexity)]
fn prepare<'buf>(mut prod: ProdIter<'buf, ConcurrentHeapRB<usize>>, mut work: WorkIter<'buf, ConcurrentHeapRB<usize>>, mut cons: ConsIter<'buf, ConcurrentHeapRB<usize>, true>)
           -> (ProdIter<'buf, ConcurrentHeapRB<usize>>, Detached<WorkIter<'buf, ConcurrentHeapRB<usize>>>, ConsIter<'buf, ConcurrentHeapRB<usize>, true>) {

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    fill_buf(&mut prod);

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE);
    assert_eq!(cons.available(), 0);

    let mut work = work.detach();

    for _ in 0..BUFFER_SIZE {
        if let Some(data) = work.get_workable() {
            *data += 1;
            unsafe { work.advance(1) };
        }
    }
    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    (prod, work, cons)
}

#[test]
fn test_work_detached_sync_index() {
    let (prod, work, cons) = ConcurrentHeapRB::default(BUFFER_SIZE + 1).split_mut();

    let (mut prod, mut work, mut cons) = prepare(prod, work, cons);

    work.sync_index();

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), BUFFER_SIZE);

    for i in 0..BUFFER_SIZE {
        assert_eq!(cons.pop().unwrap(), i + 1);
    }

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);
}

#[test]
fn test_work_detached() {
    let (prod, work, cons) = ConcurrentHeapRB::default(BUFFER_SIZE + 1).split_mut();

    let (mut prod, work, mut cons) = prepare(prod, work, cons);

    let mut work = work.attach();

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), BUFFER_SIZE);

    for i in 0..BUFFER_SIZE {
        assert_eq!(cons.pop().unwrap(), i + 1);
    }

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);
}

#[test]
fn test_work_detached_set_index() {
    let (prod, work, cons) = ConcurrentHeapRB::default(BUFFER_SIZE + 1).split_mut();

    let (mut prod, mut work, mut cons) = prepare(prod, work, cons);

    unsafe { work.set_index(work.index() - 1); }

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 1);
    assert_eq!(cons.available(), 0);

    let mut work = work.attach();

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 1);
    assert_eq!(cons.available(), BUFFER_SIZE - 1);

    for i in 0..BUFFER_SIZE - 1 {
        assert_eq!(cons.pop().unwrap(), i + 1);
    }

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 1);
    assert_eq!(cons.available(), 0);
}

#[test]
fn test_work_go_back() {
    let buf: ConcurrentHeapRB<usize> = ConcurrentHeapRB::default(BUFFER_SIZE + 1);
    let (_, work, _) = buf.split_mut();

    let mut work = work.detach();

    assert_eq!(work.index(), 0);

    unsafe { work.go_back(1); }

    assert_eq!(work.index(), BUFFER_SIZE);

    unsafe { work.advance(2); }

    assert_eq!(work.index(), 1);

    unsafe { work.go_back(1); }

    assert_eq!(work.index(), 0);
}