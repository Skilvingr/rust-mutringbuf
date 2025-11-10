use crate::{common_def, get_buf};
use mutringbuf::MRBIterator;

common_def!();

#[test]
pub fn prod_drop_test() {
    let mut buf = get_buf!(Concurrent);
    let (prod, _work, cons) = buf.split_mut();

    assert_eq!(cons.alive_iters(), 3);

    drop(prod);

    assert_eq!(cons.alive_iters(), 2);
}

#[test]
pub fn work_drop_test() {
    let mut buf = get_buf!(Concurrent);
    let (_prod, work, cons) = buf.split_mut();

    assert_eq!(cons.alive_iters(), 3);

    drop(work);

    assert_eq!(cons.alive_iters(), 2);
}

#[test]
pub fn cons_drop_test() {
    let mut buf = get_buf!(Concurrent);
    let (prod, _work, cons) = buf.split_mut();

    assert_eq!(prod.alive_iters(), 3);

    drop(cons);

    assert_eq!(prod.alive_iters(), 2);
}

#[test]
pub fn drop_everything() {
    let mut buf = get_buf!(Concurrent);
    let (prod, work, cons) = buf.split_mut();

    drop(prod);
    drop(work);
    drop(cons);
}
