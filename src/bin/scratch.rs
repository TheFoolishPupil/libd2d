#[macro_use(array)]
extern crate ndarray;
use ndarray::Axis;

fn main () {

    let mission_area: ndarray::Array2<u32> = array![[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                    [0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                    [0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                    [0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0],
                                                    [0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0],
                                                    [0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0],
                                                    [0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0],
                                                    [0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0],
                                                    [0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0],
                                                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0],
                                                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 0],
                                                    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12]];

    let view = mission_area.view();

    let (x,y) = view.split_at(Axis(1), 6);
    
    println!("{}", x[[0,0]]);
    println!("TESTING");
    println!("{}", y[[0,0]]);

    // println!("{:?}", view);
    // println!("{:?}", view.split_at(ndarray::Axis(1), 6));
}
