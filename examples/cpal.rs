#[cfg(cpal)]
fn main() {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::Relaxed;
    use std::thread;
    use cpal::{InputCallbackInfo, OutputCallbackInfo, StreamConfig};
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use mutringbuf::{ConcurrentHeapRB, MRBIterator};

    const BUF_SIZE: usize = 1000000;
    const DELAY_MS: usize = 500;
    const DECAY: f32 = 0.5;

    let host = cpal::default_host();
    let (in_dev, out_dev) = (host.default_input_device().unwrap(), host.default_output_device().unwrap());
    let (in_cfg, out_cfg) = (
        StreamConfig::from(in_dev.default_input_config().unwrap()),
        StreamConfig::from(out_dev.default_output_config().unwrap())
    );

    let delay_samples = DELAY_MS * (in_cfg.sample_rate.0 as usize / 1000);

    let buf = ConcurrentHeapRB::from(vec![0.; BUF_SIZE]);
    let (mut prod, mut work, mut cons) = buf.split_mut(vec![0f32; delay_samples]);

    let stop_worker = Arc::new(AtomicBool::new(false));
    let stop_clone = stop_worker.clone();
    let worker = thread::spawn(move || {
        while !stop_clone.load(Relaxed) {
            if let Some(((h, t), bt)) = work.get_workable_slice_exact(delay_samples) {
                let len = h.len() + t.len();
                h.swap_with_slice(&mut bt[..h.len()]);
                t.swap_with_slice(&mut bt[h.len()..]);

                for (v, w) in h.iter_mut().chain(t).zip(bt) {
                    *v *= DECAY;
                    *v += *w;
                }

                unsafe { work.advance(len) };
            }
        };

        // work was moved by thread::spawn, so, returning it here will allow to use it after call to join
        work
    });

    let in_stream = in_dev.build_input_stream(
        &in_cfg,
        move |slice: &[f32], _info: &InputCallbackInfo| {
            if prod.push_slice(slice).is_none() { println!("Input iter fell behind!"); }
        },
        move |err| { println!("INPUT ERROR: {}", err) },
        None
    ).expect("Cannot create input stream");

    let out_stream = out_dev.build_output_stream(
        &out_cfg,
        move |slice: &mut [f32], _info: &OutputCallbackInfo| {
            let len = slice.len();
            if let Some((h, t)) = cons.peek_slice(len) {
                slice[.. h.len()].copy_from_slice(h);
                slice[h.len() ..].copy_from_slice(t);

                unsafe { cons.advance(len) };
            } else {
                println!("Output iter fell behind!");
            }
        },
        move |err| { println!("OUTPUT ERROR: {}", err) },
        None
    ).expect("Cannot create output stream");

    in_stream.play().unwrap();
    out_stream.play().unwrap();

    println!("Playing for 15 seconds... ");
    thread::sleep(std::time::Duration::from_secs(15));
    stop_worker.store(true, Relaxed); // Stop worker thread

    let work = worker.join().unwrap();

    drop(in_stream); // This also drops prod
    assert!(!work.is_prod_alive());
    drop(out_stream); // This also drops cons
    assert!(!work.is_cons_alive());
    drop(work); // Drop work

    // => All iterators are now dropped, so buf gets dropped as well.
}

#[cfg(not(cpal))]
pub fn main() {}