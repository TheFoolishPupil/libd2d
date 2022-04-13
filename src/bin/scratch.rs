use ndarray::Array2;

#[derive(Debug)]
struct Area {
    grid: Array2<u32>,
}

fn main () {

    let x:[[i32;2];3] = [[8;2];3];
    println!("{:?}", x);

    let mut area = Area {
        grid: Array2::<u32>::zeros((3, 4)),
    };

    println!("{:?}", area);
    
    area.grid[[0,0]] = 1;

    println!("{:?}", area);

}
