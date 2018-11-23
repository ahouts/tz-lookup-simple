# tz-lookup-simple

Simple library to find the timezone name for geo coordinates, inspired
by [tz-lookup](https://github.com/darkskyapp/tz-lookup).

This library is a proof of concept, use at your own risk.
Also the runtime size is over 140MB... so yeah.

## Getting Started

There are two main ways to use this library. One way is to provide the timezone geojson from 
[timezone-boundary-builder](https://github.com/evansiroky/timezone-boundary-builder/)
yourself.

```rust
use tz_lookup_simple::TzLookup;
use std::fs::File;

let geo_json = File::open("./my/local/geojson_data.json").unwrap();
let tzl = TzLookup::new(geo_json).unwrap();
```

However, tz-lookup-simple also provides an inline version of the timezone data for
ease of use.

To enable it, add the following line to your Cargo.toml

**NOTE: Enabling this feature will increase your binary size by ~18MB**

```toml
tz-lookup-simple = { version = "*", features = ["inline_tzdata_complete"] }
```

Then you can use the "from_inline_complete" function to get a TzLookup.

```rust
use tz_lookup_simple::TzLookup;

let tzl = TzLookup::from_inline_complete();
```

## Updating the Inline Time Zone Data

There's a script for that! First, update the TIMEZONE_GEOJSON_URL
in "./src/update_tzdata.rs". Then run the update_tzdata example.

```bash
cargo run --release --example update_tzdata
```

Feel free to submit a PR whenever
[timezone-boundary-builder](https://github.com/evansiroky/timezone-boundary-builder/)
releases an update.
