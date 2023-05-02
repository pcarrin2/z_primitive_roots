use std::fs::File;
use std::io::prelude::*;
use std::env;
use number_theory;
use libm::floor;
use divisors::get_divisors;
use flume;
use std::thread;

fn find_first_pr(p: u64, divs: &Vec<u64>) -> u64 {
    // first, take p mod 8 to find the best order in which to check numbers
    // each of the ranges in the match statement is basically a shuffling of 
    // integers from 1 to p-1. (or in one case, p-2). don't worry, QRs get
    // filtered out later.
    let p_f = p as f64;
    let check_list: Box<dyn Iterator<Item=u64>> = match p % 8 {
        1 => {let p_over_3 = floor(p_f/3.0) as u64; 
            Box::new([2].into_iter().chain(p_over_3..p-1).chain(3..p_over_3))}
        3 => Box::new((2..p-1).rev()),
        5 => {let p_over_3 = floor(p_f/3.0) as u64; 
            Box::new((p_over_3..p-1).chain(2..p_over_3))}
        7 => Box::new([2].into_iter().chain((3..p-2).rev())),
        _ => {panic!("p is not an odd prime! p mod 8 is even!");}
    };


    for n in check_list {
        // check for QRs
        if number_theory::NumberTheory::legendre(&n, &p)==1 {
            continue;
        }

        // the order of n mod p will divide p-1 (the Carmichael lambda of p)
        let mut is_pr = true;
        for d in divs { // should export these divisors too tbh
            if number_theory::NumberTheory::exp_residue(&n, d, &p) == 1 {
                // this n is not a pr mod p
                is_pr = false;
                break;
            }
        }
        if is_pr { return n; }
    }
    return 0; // should never be called
}

fn find_all_prs(p: u64, divs: &Vec<u64>) -> Vec<u64> {
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
        //calculate divisors of p-1 only once 
        let mut divs = get_divisors(p-1);
        let prs = find_all_prs(p, &divs);
        let mut divs_times_p = divs.clone().into_iter().map(|x| x*p).collect();
        divs.push(p);
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

    for _ in 0..4 { // worker threads
       let rx = rx.clone(); 
        thread::spawn(move || {
            for msg in rx.iter() {
                worker(msg[0], msg[1]);
            }
        });
    }

    drop(rx);
    

    loop { // sender threads
        tx.send(interval).expect("error sending to worker thread");
        start += gap;
        stop += gap;
        interval = vec![start, stop];
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    /*#[test]
    fn test_check_pr_mod_psquared() {
        assert_eq!(check_pr_mod_psquared(13, 6), true);
        assert_eq!(check_pr_mod_psquared(29, 14), false);
        assert_eq!(check_pr_mod_psquared(7, 3), true);
        assert_eq!(check_pr_mod_psquared(37, 10), false);

    }

    #[test]
    fn test_find_all_prs() {
        let mut prs_5 = find_all_prs(5);
        prs_5.sort();
        let mut prs_11 = find_all_prs(11);
        prs_11.sort();
        let mut prs_7 = find_all_prs(7);
        prs_7.sort();
        let mut prs_13 = find_all_prs(13);
        prs_13.sort();
        assert_eq!(prs_5, vec![2 as u64, 3 as u64]);
        assert_eq!(prs_11, vec![2 as u64, 6 as u64, 7 as u64, 8 as u64]);
        assert_eq!(prs_7, vec![3 as u64, 5 as u64]);
        assert_eq!(prs_13, vec![2 as u64, 6 as u64, 7 as u64, 11 as u64]);
    }*/
    #[test]
    fn test_divs_psquared() {
        let p = 101 as u64;
        //calculate divisors of p-1 only once 
        let mut divs = get_divisors(p-1);
        let prs = find_all_prs(p, &divs);
        let mut divs_times_p = divs.clone().into_iter().map(|x| x*p).collect();
        divs.push(p);
        divs.push(p-1);
        divs.append(&mut divs_times_p);
        divs.sort();
        let mut canonical_divs_psquared = get_divisors(p*(p-1));
        canonical_divs_psquared.sort();
        assert_eq!(divs, canonical_divs_psquared);
    }

}
