extern crate alloc;

use crate::{common_def, get_buf};
use mutringbuf::iterators::Detached;
use mutringbuf::iterators::{ConsIter, ProdIter, WorkIter};
use mutringbuf::{MRBIterator, MutRB};

common_def!();

fn fill_buf(prod: &mut ProdIter<impl MutRB<Item = usize>>) {
    let slice = (0..BUFFER_SIZE - 1).collect::<Vec<usize>>();
    prod.push_slice(&slice);
}

#[allow(clippy::type_complexity)]
fn prepare<'buf>(
    mut prod: ProdIter<'buf, impl MutRB<Item = usize>>,
    mut work: WorkIter<'buf, impl MutRB<Item = usize>>,
    mut cons: ConsIter<'buf, impl MutRB<Item = usize>, true>,
) -> (
    ProdIter<'buf, impl MutRB<Item = usize>>,
    Detached<WorkIter<'buf, impl MutRB<Item = usize>>>,
    ConsIter<'buf, impl MutRB<Item = usize>, true>,
) {
    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    assert_eq!(work.available(), 0);
    assert_eq!(cons.available(), 0);

    fill_buf(&mut prod);

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), BUFFER_SIZE - 1);
    assert_eq!(cons.available(), 0);

    let mut work = work.detach();

    for _ in 0..BUFFER_SIZE - 1 {
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
    let mut buf = get_buf!(Concurrent);
    let (prod, work, cons) = buf.split_mut();

    let (mut prod, mut work, mut cons) = prepare(prod, work, cons);

    work.sync_index();

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
fn test_work_detached() {
    let mut buf = get_buf!(Concurrent);
    let (prod, work, cons) = buf.split_mut();

    let (mut prod, work, mut cons) = prepare(prod, work, cons);

    let mut work = work.attach();

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
fn test_work_detached_set_index() {
    let mut buf = get_buf!(Concurrent);
    let (prod, work, cons) = buf.split_mut();

    let (mut prod, mut work, mut cons) = prepare(prod, work, cons);

    unsafe {
        work.set_index(work.index() - 1);
    }

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 1);
    assert_eq!(cons.available(), 0);

    let mut work = work.attach();

    assert_eq!(prod.available(), 0);
    assert_eq!(work.available(), 1);
    assert_eq!(cons.available(), BUFFER_SIZE - 2);

    for i in 0..BUFFER_SIZE - 2 {
        assert_eq!(cons.pop().unwrap(), i + 1);
    }

    assert_eq!(prod.available(), BUFFER_SIZE - 2);
    assert_eq!(work.available(), 1);
    assert_eq!(cons.available(), 0);
}

#[test]
fn test_work_go_back() {
    let mut buf = get_buf!(Concurrent);
    let (_, work, _) = buf.split_mut();

    let mut work = work.detach();

    assert_eq!(work.index(), 0);

    unsafe {
        work.go_back(1);
    }

    assert_eq!(work.index(), BUFFER_SIZE - 1);

    unsafe {
        work.advance(2);
    }

    assert_eq!(work.index(), 1);

    unsafe {
        work.go_back(1);
    }

    assert_eq!(work.index(), 0);
}
