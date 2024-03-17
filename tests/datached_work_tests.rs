extern crate alloc;

use mutringbuf::{ConsIter, DetachedWorkIter, ConcurrentHeapRB, Iterator, ProdIter, WorkIter};

const BUFFER_SIZE: usize = 100;

fn fill_buf(prod: &mut ProdIter<ConcurrentHeapRB<usize>, usize>) {
    let slice = (0..BUFFER_SIZE).collect::<Vec<usize>>();
    prod.push_slice(&slice);
}

#[allow(clippy::type_complexity)]
fn prepare(mut prod: ProdIter<ConcurrentHeapRB<usize>, usize>, mut work: WorkIter<ConcurrentHeapRB<usize>, usize, i32>, mut cons: ConsIter<ConcurrentHeapRB<usize>, usize, true>)
           -> (ProdIter<ConcurrentHeapRB<usize>, usize>, DetachedWorkIter<ConcurrentHeapRB<usize>, usize, i32>, ConsIter<ConcurrentHeapRB<usize>, usize, true>) {

    assert_eq!(prod.available(), BUFFER_SIZE);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    fill_buf(&mut prod);

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE);
    assert_eq!(cons.available(), 0);

    let mut work = work.detach();

    for _ in 0..BUFFER_SIZE {
        if let Some((data, _)) = work.get_workable() {
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
    let (prod, work, cons) = ConcurrentHeapRB::new(BUFFER_SIZE + 1).split_mut(0);

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
    let (prod, work, cons) = ConcurrentHeapRB::new(BUFFER_SIZE + 1).split_mut(0);

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
    let (prod, work, cons) = ConcurrentHeapRB::new(BUFFER_SIZE + 1).split_mut(0);

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
    let buf: ConcurrentHeapRB<usize> = ConcurrentHeapRB::new(BUFFER_SIZE + 1);
    let (_, work, _) = buf.split_mut(0);

    let mut work = work.detach();

    assert_eq!(work.index(), 0);

    unsafe { work.go_back(1); }

    assert_eq!(work.index(), BUFFER_SIZE);

    unsafe { work.advance(2); }

    assert_eq!(work.index(), 1);

    unsafe { work.go_back(1); }

    assert_eq!(work.index(), 0);
}