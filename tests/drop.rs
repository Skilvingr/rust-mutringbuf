use mutringbuf::{ConsIter, ConcurrentHeapRB, ProdIter, WorkIter};


fn prepare() -> (ProdIter<ConcurrentHeapRB<usize>, usize>, WorkIter<ConcurrentHeapRB<usize>, usize, usize>, ConsIter<ConcurrentHeapRB<usize>, usize, true>) {
    let buf = ConcurrentHeapRB::from(vec![0; 10]);
    buf.split_mut(0)
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