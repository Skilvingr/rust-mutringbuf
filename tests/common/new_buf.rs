use crate::common_def;
use mutringbuf::iterators::ProdIter;
use mutringbuf::{MRBIterator, MutRB};
use std::slice::Iter;

common_def!(buf);

fn test_buf(v: Iter<usize>, mut prod: ProdIter<impl MutRB<Item = usize>>) {
    #[cfg(not(feature = "vmem"))]
    let b = {
        let (h, t) = prod.get_workable_slice_avail().unwrap();
        h.iter().chain(t.iter()).map(|e| *e).collect::<Vec<usize>>()
    };

    #[cfg(feature = "vmem")]
    let b = prod.get_workable_slice_avail().unwrap();

    for (be, ve) in b.iter().zip(v) {
        assert_eq!(*ve, *be);
    }
}

#[test]
fn test_new_buf() {
    let v: [usize; BUFFER_SIZE] = (0..BUFFER_SIZE).collect::<Vec<usize>>().try_into().unwrap();

    #[cfg(feature = "alloc")]
    {
        use mutringbuf::{ConcurrentHeapRB, HeapSplit, LocalHeapRB};
        let (prod, _) = ConcurrentHeapRB::from(v.clone().to_vec()).split();
        test_buf(v.iter(), prod);

        let (prod, _) = LocalHeapRB::from(v.clone().to_vec()).split();
        test_buf(v.iter(), prod);
    }

    #[cfg(not(feature = "vmem"))]
    {
        use mutringbuf::{ConcurrentStackRB, LocalStackRB, StackSplit};

        let mut buf = ConcurrentStackRB::from(v.clone());
        let (prod, _) = buf.split();
        test_buf(v.iter(), prod);

        let mut buf = LocalStackRB::from(v.clone());
        let (prod, _) = buf.split();
        test_buf(v.iter(), prod);
    }
}
