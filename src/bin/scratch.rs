#[macro_use(array)]
extern crate ndarray;
use ndarray::{Axis, concatenate};


fn main () {

    let minion_count = 3;

    let arr = array![[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0],
                    [0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0]];

    // let arr = array![[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //                 [0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //                 [0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //                 [0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0],
    //                 [0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0],
    //                 [0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0],
    //                 [0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0],
    //                 [0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0],
    //                 [0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0],
    //                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0],
    //                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 0],
    //                 [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12],
    //                 [13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //                 [0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]];

    let (axis, axis_size) = arr.shape().iter().enumerate().max_by_key(|(_,v)| *v).unwrap();
    println!("Largest axis: {:?} with size: {:?}", axis, axis_size);

    let splits = axis_size / minion_count;
    let rem = axis_size % minion_count; 

    println!("splits: {:?}. rem: {:?}", splits, rem);

    let mut split = arr.axis_chunks_iter(Axis(axis), splits);
    if rem > 0 {
        let last1 = split.next_back().unwrap(); // `n-1`th element
        let last2 = split.next_back().unwrap(); // `n-2`th element

        let split = split.map(|x| x.to_owned());
        let joint = concatenate(Axis(axis), &[last2, last1]).unwrap();

        let split = split.chain([joint]);
        let mut dim = [0,0];

        for i in split {
            println!("COOR:{:?} SPLIT:{:?}\n", dim, i);
            dim[axis] += splits;
        }

    } else {

        let mut dim = [0,0];

        for i in split {
            println!("COOR:{:?} SPLIT:{:?}\n", dim, i);
            dim[axis] += splits;
        }
        
    }

    // Use this for minion to traverse array.
    // for i in arr.indexed_iter() {
    //     println!("{:?}", i);
    // }

}
