rust-baseball
===

A pure Rust baseball data aggregation and analytics library. 
Currently only does some aggregation from the MLB GameDay XML files

### Documentation

TODO

### Usage

```toml
[dependencies]
baseball = "0.1"
```
### Prior Art

[baseballr](https://github.com/BillPetti/baseballr) by Bill Petti

[pitchrx](https://github.com/cpsievert/pitchRx) by Carson Sievert

### Motivation

Building a baseball data engine in Rust will enable everyday fans to perform data-intensive workloads, as well as efficient data gathering. Ambitiously, aiming for a baseball data platform that will rival what MLB clubs have internally, from an analytics perspective. Clearly, MLB clubs will have access to more, and likely better, data.

This project is also a learning project for the author and should change a lot as the author better hones his Rust skills.

### Features
* Parallel out of the box. Player bios are memoized (cached) once they've been downloaded once, drastically reducing the number of network calls.
* Captures historical player weight info
* Flattens out all the data and serializes to an easy to use CSV file.

### Roadmap

* Tools to gather data from the GameDay xml files, for all levels of affiliated baseball
* Tools to gather statcast data, as well as calculation for spin efficiency
* Incoporate the Rust retrosheet parser and try to align the data to the GameDay and StatCast data sets. Hopefully will be able to use the existing code base
* Export flattened (denormalized) games to CSV (Ideally, there should be an option to split out into 2 files, one for metadata, one for play-by-play, or one big flat file)