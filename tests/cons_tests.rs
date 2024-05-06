use mutringbuf::{ConcurrentHeapRB, ConcurrentStackRB, MRBIterator, ProdIter};
use mutringbuf::ring_buffer::variants::ring_buffer_trait::MutRB;

const BUFFER_SIZE: usize = 100;

macro_rules! get_buf {
    (Local, Stack) => { LocalStackRB::from([0; BUFFER_SIZE + 1]) };
    (Local, Heap) => { LocalHeapRB::from(vec![0; BUFFER_SIZE + 1]) };
    (Concurrent, Stack) => { ConcurrentStackRB::from([0; BUFFER_SIZE + 1]) };
    (Concurrent, Heap) => { ConcurrentHeapRB::from(vec![0; BUFFER_SIZE + 1]) };
}

fn fill_buf<B: MutRB<Item = usize>>(prod: &mut ProdIter<B>, count: usize) {
    for i in 0..count {
        let _ = prod.push(i);
    }
}

#[test]
fn test_pop_exact() {
    let buf = get_buf!(Concurrent, Stack);
    let (mut prod, mut cons) = buf.split();

    assert!(cons.pop().is_none());

    fill_buf(&mut prod, BUFFER_SIZE);

    for i in 0..BUFFER_SIZE {
        assert_eq!(cons.pop().unwrap(), i);
    }

    assert!(cons.pop().is_none());

    fill_buf(&mut prod, BUFFER_SIZE);

    for i in 0..BUFFER_SIZE {
        assert_eq!(cons.pop().unwrap(), i);
    }

    assert!(cons.pop().is_none());
}

#[test]
fn test_pop_ref_exact() {
    let (mut prod, mut cons) = ConcurrentHeapRB::new(BUFFER_SIZE + 1).split();

    fill_buf(&mut prod, BUFFER_SIZE);

    for i in 0..BUFFER_SIZE {
        assert_eq!(cons.peek_ref().unwrap(), &i);
        unsafe { cons.advance(1) };
    }

    assert!(cons.pop().is_none());
}

#[test]
fn test_pop_slice_exact() {
    let (mut prod, mut cons) = ConcurrentHeapRB::new(BUFFER_SIZE + 1).split();

    fill_buf(&mut prod, BUFFER_SIZE);

    let (head, tail) = cons.peek_slice(BUFFER_SIZE).unwrap();
    assert!(!head.is_empty() || !tail.is_empty());

    for (p, i) in [head, tail].concat().iter().zip(0..BUFFER_SIZE) {
        assert_eq!(p, &i);
    }
    unsafe { cons.advance(BUFFER_SIZE) };

    assert!(cons.pop().is_none());
}

#[test]
fn test_pop_avail_nw_exact() {
    let (mut prod, mut cons) = ConcurrentHeapRB::new(BUFFER_SIZE + 1).split();

    fill_buf(&mut prod, BUFFER_SIZE);

    let (head, tail) = cons.peek_available().unwrap();
    assert_eq!(head.len() + tail.len(), BUFFER_SIZE);

    for (p, i) in [head, tail].concat().iter().zip(0..BUFFER_SIZE) {
        assert_eq!(p, &i);
    }
    unsafe { cons.advance(BUFFER_SIZE) };

    assert!(cons.pop().is_none());
}

#[test]
fn test_pop_slice_seam() {
    let (mut prod, mut cons) = ConcurrentHeapRB::new(BUFFER_SIZE + 1).split();

    fill_buf(&mut prod, BUFFER_SIZE / 2);

    let (head, tail) = cons.peek_available().unwrap();
    assert_eq!(head.len(), BUFFER_SIZE / 2);
    assert!(tail.is_empty());
    unsafe { cons.advance(BUFFER_SIZE / 2) };

    fill_buf(&mut prod, BUFFER_SIZE);

    let (head, tail) = cons.peek_available().unwrap();
    assert_eq!(head.len(), BUFFER_SIZE / 2 + 1);
    assert_eq!(tail.len(), BUFFER_SIZE / 2 - 1);

    for (p, i) in [head, tail].concat().iter().zip(0..BUFFER_SIZE) {
        assert_eq!(p, &i);
    }
    unsafe { cons.advance(BUFFER_SIZE) };

    assert!(cons.pop().is_none());
}

#[test]
fn test_pop_slice_copy() {
    let (mut prod, mut cons) = ConcurrentHeapRB::new(BUFFER_SIZE + 1).split();

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