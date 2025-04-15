use mutringbuf::{MRBIterator, MutRB};
use mutringbuf::iterators::ProdIter;
use crate::{common_def, get_buf};

common_def!();

fn fill_buf<B: MutRB<Item=usize>>(prod: &mut ProdIter<B>, count: usize) {
    for i in 0..count {
        let _ = prod.push(i);
    }
}

#[test]
fn test_pop_exact() {
    let buf = get_buf!(Concurrent);
    let (mut prod, mut cons) = buf.split();

    assert!(cons.pop().is_none());

    fill_buf(&mut prod, BUFFER_SIZE - 1);

    for i in 0..BUFFER_SIZE - 1 {
        assert_eq!(*cons.peek_ref().unwrap(), i);
        unsafe { cons.advance(1); }
    }

    assert!(cons.peek_ref().is_none());
    unsafe { cons.advance(1); }

    fill_buf(&mut prod, BUFFER_SIZE - 1);

    for i in 0..BUFFER_SIZE - 1 {
        assert_eq!(*cons.peek_ref().unwrap(), i);
        unsafe { cons.advance(1); }
    }

    assert!(cons.pop().is_none());
}

#[test]
fn test_pop_ref_exact() {
    let (mut prod, mut cons) = get_buf!(Concurrent).split();

    fill_buf(&mut prod, BUFFER_SIZE - 1);

    for i in 0..BUFFER_SIZE - 1 {
        assert_eq!(cons.peek_ref().unwrap(), &i);
        unsafe { cons.advance(1) };
    }

    assert!(cons.pop().is_none());
}

#[test]
fn test_pop_slice_exact() {
    let (mut prod, mut cons) = get_buf!(Concurrent).split();

    fill_buf(&mut prod, BUFFER_SIZE - 1);

    let res = cons.peek_slice(BUFFER_SIZE - 1).unwrap();
    assert!(!res.is_empty());

    for (p, i) in res.iter().zip(0..BUFFER_SIZE - 1) {
        assert_eq!(p, &i);
    }
    unsafe { cons.advance(BUFFER_SIZE - 1) };

    assert!(cons.pop().is_none());
}

#[test]
fn test_pop_avail_nw_exact() {
    let (mut prod, mut cons) = get_buf!(Concurrent).split();

    fill_buf(&mut prod, BUFFER_SIZE - 1);

    let res = cons.peek_available().unwrap();
    assert_eq!(res.len(), BUFFER_SIZE - 1);

    for (p, i) in res.iter().zip(0..BUFFER_SIZE - 1) {
        assert_eq!(p, &i);
    }
    unsafe { cons.advance(BUFFER_SIZE - 1) };

    assert!(cons.pop().is_none());
}

#[test]
fn test_pop_slice_seam() {
    let (mut prod, mut cons) = get_buf!(Concurrent).split();

    fill_buf(&mut prod, BUFFER_SIZE / 2);

    let res = cons.peek_available().unwrap();
    assert_eq!(res.len(), BUFFER_SIZE / 2);
    unsafe { cons.advance(BUFFER_SIZE / 2) };

    fill_buf(&mut prod, BUFFER_SIZE);

    let res = cons.peek_available().unwrap();
    assert_eq!(res.len(), BUFFER_SIZE - 1);

    for (p, i) in res.iter().zip(0..BUFFER_SIZE - 1) {
        assert_eq!(p, &i);
    }
    unsafe { cons.advance(BUFFER_SIZE - 1) };

    assert!(cons.pop().is_none());
}

#[test]
fn test_pop_slice_copy() {
    let (mut prod, mut cons) = get_buf!(Concurrent).split();

    fill_buf(&mut prod, BUFFER_SIZE / 2);

    let mut vec = vec![0; BUFFER_SIZE / 2];

    assert!(cons.copy_slice(&mut vec).is_some());
    assert!(cons.copy_slice(&mut vec).is_none());

    fill_buf(&mut prod, BUFFER_SIZE / 2);

    assert!(cons.clone_slice(&mut vec).is_some());
    assert!(cons.clone_slice(&mut vec).is_none());

    let _ = prod.push(1);

    let mut dst = 0;

    assert!(cons.copy_item(&mut dst).is_some());
    assert!(cons.copy_item(&mut dst).is_none());

    let _ = prod.push(1);

    assert!(cons.clone_item(&mut dst).is_some());
    assert!(cons.clone_item(&mut dst).is_none());
}