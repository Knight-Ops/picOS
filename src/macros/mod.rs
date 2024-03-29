#[macro_export]
macro_rules! spinlock {
    ($i:expr) => {
        $i.lock().borrow().as_ref().unwrap()
    };
}

#[macro_export]
macro_rules! mut_spinlock {
    ($i:expr) => {
        $i.lock().borrow_mut().as_mut().unwrap()
    };
}
