extern crate alloc;

#[cfg(feature = "async")]
fn main() {
    use mutringbuf::ConcurrentStackRB;
    use mutringbuf::iterators::async_iterators::AsyncIterator;

    const BUFFER_SIZE: usize = 300;

    // Heap buffer is also possible:
    // let buf = ConcurrentHeapRB::from(vec![0; BUFFER_SIZE + 1]);
    let mut buf = ConcurrentStackRB::from([0; BUFFER_SIZE + 1]);
    let (
        mut as_prod,
        mut as_work,
        mut as_cons
    ) = buf.split_mut_async();


    tokio_test::block_on(async {
        as_prod.push(1).await;

        if let Some(res) = as_work.get_workable().await {
            *res += 1;
            unsafe { as_work.advance(1); }
        }

        let res = as_cons.peek_ref().await.unwrap();
        assert_eq!(res, &2);
        unsafe { as_cons.advance(1); }

        let slice: Vec<i32> = (0..BUFFER_SIZE as i32 /2).collect();
        as_prod.push_slice(&slice).await;

        if let Some((h, t)) = as_work.get_workable_slice_avail().await {
            let len = h.len() + t.len();

            for x in h.iter_mut().chain(t) {
                *x += 1;
            }

            unsafe { as_work.advance(len); }
        }

        if let Some((h, t)) = as_cons.peek_available().await {
            for (x, y) in h.iter().chain(t).zip(&slice) {
                assert_eq!(*x, y + 1);
            }
        }

        drop(as_prod);
        assert!(!as_cons.is_prod_alive());
        drop(as_work);
        assert!(!as_cons.is_work_alive());
    });
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("To run this example, please, use the following command:");
    println!("cargo run --example simple_async --features async");
}