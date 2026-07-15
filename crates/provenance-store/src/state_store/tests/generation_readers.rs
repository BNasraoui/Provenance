use super::seeded_requirement_store;
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn public_shard_reader_waits_for_generation() {
    let (_dir, store, scope) = seeded_requirement_store();
    let writer = store.clone();
    let (staged_tx, staged_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        writer
            .write_transaction(|_| {
                staged_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            })
            .unwrap();
    });
    staged_rx.recv().unwrap();
    let (read_tx, read_rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        read_tx
            .send(store.list_requirements(&scope).unwrap())
            .unwrap();
    });

    assert!(read_rx.recv_timeout(Duration::from_millis(100)).is_err());
    release_tx.send(()).unwrap();
    assert_eq!(
        read_rx.recv_timeout(Duration::from_secs(2)).unwrap().len(),
        1
    );
    handle.join().unwrap();
    reader.join().unwrap();
}

#[test]
fn manifest_reader_waits_for_generation() {
    let (_dir, store, _) = seeded_requirement_store();
    let writer = store.clone();
    let (staged_tx, staged_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        writer.write_transaction(|_| {
            staged_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            Ok(())
        })
    });
    staged_rx.recv().unwrap();
    let (read_tx, read_rx) = mpsc::channel();
    let reader = std::thread::spawn(move || read_tx.send(store.manifest().unwrap()).unwrap());

    assert!(read_rx.recv_timeout(Duration::from_millis(100)).is_err());
    release_tx.send(()).unwrap();
    read_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    handle.join().unwrap().unwrap();
    reader.join().unwrap();
}

#[test]
fn scope_directory_reader_waits_for_generation() {
    let (_dir, store, _) = seeded_requirement_store();
    let writer = store.clone();
    let (staged_tx, staged_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        writer.write_transaction(|_| {
            staged_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            Ok(())
        })
    });
    staged_rx.recv().unwrap();
    let (read_tx, read_rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        read_tx
            .send(store.list_scope_directories().unwrap())
            .unwrap();
    });

    assert!(read_rx.recv_timeout(Duration::from_millis(100)).is_err());
    release_tx.send(()).unwrap();
    read_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    handle.join().unwrap().unwrap();
    reader.join().unwrap();
}
