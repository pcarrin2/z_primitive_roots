use std::fs::File;
use std::io::prelude::*;
use std::env;
use number_theory;
use libm::floor;
use divisors::get_divisors;
use flume;
use std::thread;

// This program finds z-primitive roots of primes from a given starting point.
// z-primitive roots mod p are primitive roots mod p that aren't primitive roots
// mod p^2. Zhuo Zhang conjectures that the proportion of primes with z-primitive
// roots goes to Artin's constant as we consider larger primes.
//
// USAGE: ./z_primitive_roots START INCREMENT
//      START: the nth prime to begin at
//      INCREMENT: how many rows to write per output file (one row per prime)
// The program writes .tsv output files to its working directory.
// To kill this program, you have to press Ctrl-C (or equivalent abort on Windows).
//
// This program's number of threads is hardcoded for now, search "threads" in all caps
// to modify that number -- then recompile.

fn find_first_pr(p: u64, divs: &Vec<u64>) -> u64 {
    // p is a given prime, divs is a vector of divisors of p-1.
    // (we could calculate divs here, but we'll need to use it in a different
    // function, and double-calculating it is horrible for performance.)
    let check_list: Box<dyn Iterator<Item=u64>> = match p % 8 {
        1 => {
            let p_f = p as f64;
            let p_over_3 = floor(p_f/3.0) as u64; 
            Box::new([2].into_iter().chain(p_over_3..p-1).chain(3..p_over_3))
        }
        3 => Box::new((2..p-1).rev()),
        5 => {
            let p_f = p as f64;
            let p_over_3 = floor(p_f/3.0) as u64; 
            Box::new((p_over_3..p-1).chain(2..p_over_3))
        }
        7 => Box::new([2].into_iter().chain((3..p-2).rev())),
        _ => {panic!("p is not an odd prime! p mod 8 is even!");}
    };


    for n in check_list {
        if number_theory::NumberTheory::legendre(&n, &p)==1 {
            continue;
        }

        // the order of n mod p will divide p-1 (the Carmichael lambda of p)
        let mut is_pr = true;
        for d in divs {
            if number_theory::NumberTheory::exp_residue(&n, d, &p) == 1 {
                is_pr = false;
                break;
            }
        }
        if is_pr { return n; }
    }
    return 0; // should never be called
}

fn find_all_prs(p: u64, divs: &Vec<u64>) -> Vec<u64> {
    // p is a given prime, divs is a vector of divisors of p-1.
    let g = find_first_pr(p, divs);
    // raise g to all k where gcd(k, p-1) = 1
    let mut all_prs = vec![];
    for k in 1..p {
        if number_theory::NumberTheory::gcd(&k, &(p-1)) == 1 {
            let new_pr = number_theory::NumberTheory::exp_residue(&g, &k, &p);
            all_prs.push(new_pr);
        }   
    }

    return all_prs;
}

fn check_pr_mod_psquared(p: u64, n:u64, divs: &Vec<u64>) -> bool {
    // p is a prime, n is a PR mod p, divs are divisors of p(p-1).
    // returns true if n is a PR mod p^2 and false otherwise.
    for d in divs {
        if number_theory::NumberTheory::exp_residue(&n, d, &(p.pow(2))) == 1 {
            return false;
        }
    }
    return true;
}

fn worker(start: u64, stop: u64) {
    // given a range, writes zpr info in that range to a file

    let filename = format!("zprs_{}_{}.tsv", start, stop);
    let mut f = File::create(filename).expect("opening file failed");
    write!(f, "p\tzprs\tn_zprs\n").expect("write error");

    for i in start..stop {
        let p = number_theory::NumberTheory::nth_prime(&(i)).unwrap();
        // calculate divisors of p-1 only once 
        let mut divs = get_divisors(p-1);
        let prs = find_all_prs(p, &divs);
        let mut divs_times_p = divs.clone().into_iter().map(|x| x*p).collect();
        divs.push(p); // adding elements to divs so it lists the divisors of p(p-1)
        divs.push(p-1);
        divs.append(&mut divs_times_p);

        let mut z_prs = vec![];
        for n in prs {
            if check_pr_mod_psquared(p, n, &divs) == false {
                z_prs.push(n);
            }
        }

        write!(f, "{:?}\t{:?}\t{}\n", p, z_prs, z_prs.len()).expect("write error");
    }
}

fn main() {
    let mut start: u64 = env::args().nth(1).unwrap().parse().unwrap();
    let gap: u64 = env::args().nth(2).unwrap().parse().unwrap();
    let mut stop = start + gap;
    let mut interval = vec![start, stop];

    

    let (tx, rx) = flume::bounded::<Vec<u64>>(0); // rendezvous channel

    for _ in 0..4 { // 4 is number of THREADS -- replace with something else
       let rx = rx.clone(); 
        thread::spawn(move || {
            for msg in rx.iter() {
                worker(msg[0], msg[1]);
            }
        });
    }

    drop(rx);
    

    loop { // sender
        tx.send(interval).expect("error sending to worker thread");
        start += gap;
        stop += gap;
        interval = vec![start, stop];
    }
}

