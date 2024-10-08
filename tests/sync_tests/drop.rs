use mutringbuf::{ConcurrentHeapRB, ConsIter, HeapSplit, ProdIter, WorkIter};
use mutringbuf::MRBIterator;

const BUF_LEN: usize = 100;

fn prepare<'buf>() -> (ProdIter<'buf, ConcurrentHeapRB<usize>>, WorkIter<'buf, ConcurrentHeapRB<usize>>, ConsIter<'buf, ConcurrentHeapRB<usize>, true>) {
    let buf = ConcurrentHeapRB::from(vec![0; BUF_LEN + 1]);
    buf.split_mut()
}

#[test]
pub fn prod_drop_test() {
    let (prod, work, cons) = prepare();

    assert!(work.is_prod_alive());
    assert!(cons.is_work_alive());
    assert!(work.is_cons_alive());

    drop(prod);

    assert!(!work.is_prod_alive());
    assert!(cons.is_work_alive());
    assert!(work.is_cons_alive());
}

#[test]
pub fn work_drop_test() {
    let (prod, work, cons) = prepare();

    assert!(cons.is_prod_alive());
    assert!(cons.is_work_alive());
    assert!(prod.is_cons_alive());

    drop(work);

    assert!(cons.is_prod_alive());
    assert!(!cons.is_work_alive());
    assert!(prod.is_cons_alive());
}

#[test]
pub fn cons_drop_test() {
    let (prod, work, cons) = prepare();

    assert!(work.is_prod_alive());
    assert!(prod.is_work_alive());
    assert!(prod.is_cons_alive());

    drop(cons);

    assert!(work.is_prod_alive());
    assert!(prod.is_work_alive());
    assert!(!prod.is_cons_alive());
}

#[test]
pub fn drop_everything() {
    let (prod, work, cons) = prepare();

    drop(prod);
    drop(work);
    drop(cons);
}