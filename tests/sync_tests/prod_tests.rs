extern crate alloc;

use mutringbuf::{MRBIterator};
use crate::{common_def, get_buf};

common_def!();

#[test]
fn test_push_one_by_one() {
    let mut buf = get_buf!(Concurrent);
    let (mut prod, _cons) = buf.split();

    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    for i in 0..BUFFER_SIZE - 1 {
        assert!(prod.push(i).is_ok());
    }
    assert_eq!(prod.available(), 0);
    assert!(prod.push(1).is_err());
}

#[test]
fn test_push_slice() {
    let mut buf = get_buf!(Concurrent);
    let (mut prod, _cons) = buf.split();

    let half_slice = (0..BUFFER_SIZE / 2 - 1).collect::<Vec<usize>>();
    let slice = (0..BUFFER_SIZE).collect::<Vec<usize>>();

    assert_eq!(prod.available(), BUFFER_SIZE - 1);

    assert!(prod.push_slice(&half_slice).is_some());

    assert_eq!(prod.available(), BUFFER_SIZE / 2);

    assert!(prod.push_slice(&slice).is_none());

    assert!(prod.push_slice(&half_slice).is_some());

    assert_eq!(prod.available(), 1);
}

#[test]
#[allow(clippy::unnecessary_cast)]
fn test_push_mut_ref_init() {
    let mut buf = get_buf!(Concurrent);
    let (mut prod, mut cons) = buf.split();
    
    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    for i in 0..BUFFER_SIZE - 1 {
        let next = prod.get_next_item_mut_init().unwrap() as *mut usize;

        unsafe {
            next.write(i);
            prod.advance(1);
        }
    }
    assert_eq!(prod.available(), 0);
    assert!(prod.push(1).is_err());

    for i in 0..BUFFER_SIZE - 1 {
        assert_eq!(cons.pop(), Some(i));
    }
}

#[test]
#[allow(clippy::unnecessary_cast)]
fn test_push_mut_ref() {
    let mut buf = get_buf!(Concurrent);
    let (mut prod, mut cons) = buf.split();
    
    assert_eq!(prod.available(), BUFFER_SIZE - 1);
    for i in 0..BUFFER_SIZE - 1 {
        unsafe {

            let next = prod.get_next_item_mut().unwrap() as *mut usize;

            next.write(i);
            prod.advance(1);
        }
    }
    assert_eq!(prod.available(), 0);
    assert!(prod.push(1).is_err());

    for i in 0..BUFFER_SIZE - 1 {
        assert_eq!(cons.pop(), Some(i));
    }
}