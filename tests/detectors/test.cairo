use debug::PrintTrait;

fn main() {
    let a:felt252 = 0x800000000000011000000000000000000000000000000000000000000000000 + 1;
    let b:felt252 = 0-1;
    a.print();
    b.print();

}