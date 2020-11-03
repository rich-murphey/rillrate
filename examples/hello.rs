use anyhow::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Error> {
    rill::install()?;
    rill::bind_all(&[&module_1::RILL, &module_2::RILL]);
    loop {
        module_1::work();
        module_2::work();
        thread::sleep(Duration::from_millis(10));
    }
}

mod module_1 {
    rill::attach_logger!();

    pub fn work() {
        rill::log!("work module_1 called".into());
    }
}

mod module_2 {
    rill::attach_logger!();

    pub fn work() {
        rill::log!("work module_2 called".into());
    }
}
