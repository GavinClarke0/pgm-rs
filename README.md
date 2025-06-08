## PGM-RS
---

A implementation of the PGM based u64 key indexes described in https://pgm.di.unipi.it/docs/cpp-reference/ in rust with zero copy serialization support. Currently only little endian architectures are supported.

## Examples


#### Standard Copy Version 
```
// Build index from sorted list of u64
let keys: Vec<u64> = (0..10000).step_by(7).collect();
let pgm = PGMIndex::build(&keys, 64).unwrap();

// Serialize index to bytes 
let bytes = pgm.to_bytes().expect("serialize failed");

// Deserialize index from bytes to 
let restored = PGMIndex::from_bytes(&bytes).expect("full deserialize failed");

let key = 9876;
let (lo, hi) = restored.search(key);

// Search original list for index ranges the value may be contained in. 
let found = keys[lo..=hi].binary_search(&key).ok();
```


#### Zero Copy Version 
```
// Build index from sorted list of u64
let keys: Vec<u64> = (0..10000).step_by(7).collect();
let pgm = PGMIndex::build(&keys, 64).unwrap();

// Serialize index to bytes 
let bytes = pgm.to_bytes().expect("serialize failed");

let archived = PGMIndex::as_archived(&bytes).expect("zero-copy deserialize failed");
let key = 1000;
let (lo, hi) = archived.search(key);

let key = 9876;
let (lo, hi) = restored.search(key);

// Search original list for index ranges the value may be contained in. 
let found = keys[lo..=hi].binary_search(&key).ok();
```