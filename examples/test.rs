use crobber::*;

fn main() {
    extern "C" fn counter(mut crob: RawCrob, data: usize) -> ! {
        println!("counter: Got init {data}");
        loop {
            for i in 0.. {
                println!("counter: Send {data}");
                let resp = crob.call(i);
                println!("counter: Got {resp}");
            }
        }
    }

    let mut stack = [0; 4096];
    let mut crob = RawCrob::new(&mut stack, counter);
    for i in 100..110 {
        println!("main: Send {i}");
        let res = crob.call(i);
        println!("main: Got {res}");
        assert_eq!(i, res + 100);
    }
}
