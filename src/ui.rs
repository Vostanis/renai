use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub fn single_pb(length: u64) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [ {bar:50} ] {pos}/{len} {msg} {spinner}")
            .unwrap()
            .progress_chars("#|-"),
    );
    pb
}

pub fn success_tracker_pb(length: u64) -> MultiProgress {
    let m = MultiProgress::new();
    let total_bar = m.add(ProgressBar::new(length));
    let success_bar = m.add(ProgressBar::new(length));
    let failure_bar = m.add(ProgressBar::new(length));

    total_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{wide_bar:.cyan/blue}] {pos}/{len} {msg} {spinner}")
            .unwrap(),
    );
    success_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{wide_bar:.green/blue}] {pos}/{len}")
            .unwrap(),
    );
    failure_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{wide_bar:.red/blue}] {pos}/{len}")
            .unwrap(),
    );

    m
}
