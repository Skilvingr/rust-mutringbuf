extern crate alloc;

use mutringbuf::{ConcurrentHeapRB, MRBIterator};

const BUFFER_SIZE: usize = 100;

#[test]
fn test_push_one_by_one() {
    let (mut prod, _cons) = ConcurrentHeapRB::new(BUFFER_SIZE + 1).split();

    assert_eq!(prod.available(), BUFFER_SIZE);
    for i in 0..BUFFER_SIZE {
        assert!(prod.push(i).is_ok());
    }
    assert_eq!(prod.available(), 0);
    assert!(prod.push(1).is_err());
}

#[test]
fn test_push_slice() {
    let (mut prod, _cons) = ConcurrentHeapRB::new(BUFFER_SIZE + 1).split();

    let half_slice = (0..BUFFER_SIZE / 2).collect::<Vec<usize>>();
    let slice = (0..BUFFER_SIZE).collect::<Vec<usize>>();

    assert_eq!(prod.available(), BUFFER_SIZE);

    assert!(prod.push_slice(&half_slice).is_some());

    assert_eq!(prod.available(), BUFFER_SIZE / 2);

    assert!(prod.push_slice(&slice).is_none());

    assert!(prod.push_slice(&half_slice).is_some());

    assert_eq!(prod.available(), 0);
}

#[test]
#[allow(clippy::unnecessary_cast)]
fn test_push_mut_ref_init() {
    let (mut prod, mut cons) = ConcurrentHeapRB::new(BUFFER_SIZE + 1).split();

    assert_eq!(prod.available(), BUFFER_SIZE);
    for i in 0..BUFFER_SIZE {
        let next = prod.get_next_item_mut_init().unwrap() as *mut usize;

        unsafe {
            next.write(i);
            prod.advance(1);
        }
    }
    assert_eq!(prod.available(), 0);
    assert!(prod.push(1).is_err());

    for i in 0..BUFFER_SIZE {
        assert_eq!(cons.pop(), Some(i));
    }
}

#[test]
#[allow(clippy::unnecessary_cast)]
fn test_push_mut_ref() {
    let (mut prod, mut cons) = ConcurrentHeapRB::default(BUFFER_SIZE + 1).split();

    assert_eq!(prod.available(), BUFFER_SIZE);
    for i in 0..BUFFER_SIZE {
        unsafe {

            let next = prod.get_next_item_mut().unwrap() as *mut usize;

            next.write(i);
            prod.advance(1);
        }
    }
    assert_eq!(prod.available(), 0);
    assert!(prod.push(1).is_err());

    for i in 0..BUFFER_SIZE {
        assert_eq!(cons.pop(), Some(i));
    }
}