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

### Motivation

Building a baseball data engine in Rust will enable everyday fans to perform data-intensive workloads, as well as efficient data gathering. Ambitiously, aiming for a baseball data platform that will rival what MLB clubs have internally, from an analytical capability perspective.

This project is also a learning project for the author and should change a lot as the author better hones his Rust skills.

### Roadmap

* Tools to gather data from the GameDay xml files, for all levels of affiliated baseball
* Tools to gather statcast data, as well as calculation for spin efficiency
* Incoporate the Rust retrosheet parser and try to align the data to the GameDay and StatCast data sets. Hopefully will be able to use the existing code base
* Export flattened (denormalized) games to CSV 