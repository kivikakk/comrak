use remstack::RemStack;
use std::ptr;

#[test]
fn push_read() {
    let rs: RemStack<_> = vec![1, 2, 3].into();

    assert_eq!(rs[0], 1);
    assert_eq!(rs[1], 2);
    assert_eq!(rs[2], 3);
}

#[test]
fn remove() {
    let mut rs: RemStack<_> = vec![1, 2, 3].into();
    rs.remove(1);

    assert!(!ptr::eq(&rs[0], &rs[1]));
    assert!(!ptr::eq(&rs[0], &rs[2]));
    assert!(ptr::eq(&rs[1], &rs[2]));
}

#[test]
fn debug() {
    let mut rs: RemStack<_> = vec![1, 2, 3].into();
    assert_eq!(format!("{:?}", rs), "RemStack [1, 2, 3]");

    rs.remove(1);
    assert_eq!(format!("{:?}", rs), "RemStack [1, _, 3]");

    rs.remove(0);
    assert_eq!(format!("{:?}", rs), "RemStack [_, _, 3]");

    rs.remove(2);
    assert_eq!(format!("{:?}", rs), "RemStack []");

    rs.push(4);
    assert_eq!(format!("{:?}", rs), "RemStack [4]");
}
