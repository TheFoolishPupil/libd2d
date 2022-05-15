use ndarray::Array;
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Uniform;


fn main() {
    let a = Array::random((15, 6), Uniform::new(0, 2));
    println!("{}", a);
}