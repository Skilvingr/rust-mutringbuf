use mutringbuf::MRBIterator;
use crate::{common_def, get_buf};

common_def!();

#[test]
pub fn prod_drop_test() {
    let mut buf = get_buf!(Concurrent);
    let (prod, work, cons) = buf.split_mut();
    
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
    let mut buf = get_buf!(Concurrent);
    let (prod, work, cons) = buf.split_mut();

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
    let mut buf = get_buf!(Concurrent);
    let (prod, work, cons) = buf.split_mut();

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
    let mut buf = get_buf!(Concurrent);
    let (prod, work, cons) = buf.split_mut();

    drop(prod);
    drop(work);
    drop(cons);
}