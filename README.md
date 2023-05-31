# z_primitive_roots
Find primitive roots mod p that aren't primitive roots mod p^2, in Rust.

## To run:
Clone to your machine, compile, and run `./z_primitive_roots [START] [INCREMENT]`, where START is the nth prime to start at, and INCREMENT is the number of primes per file.

`z_primitive_roots` will generate a bunch of TSV files in its working directory, each with three columns: the prime tested, the list of *z*-primitive roots, and the number of *z*-primitive roots.
These files can then be analyzed or concatenated with any mainstream statistical software or with Bash line editing.
